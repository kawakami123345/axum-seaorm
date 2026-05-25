use crate::{UserContext, error::UseCaseError};
use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityId, EntityTypeName, EntityUid,
    PartialEntities, PartialEntityUid, PartialRequest, Policy as CedarPolicy, PolicySet, Request,
    RestrictedExpression, Schema,
    pst::{self, BinaryOp, Clause, Expr, Literal, UnaryOp, Var},
};
use cedar_schema_macros::cedar_schema_consts;
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::LazyLock,
};

// 黒魔術: cedarschemaからconst作成
cedar_schema_consts!("policies/schema.cedarschema");

const POLICY_FILE: &str = "policies/book.cedar";
const SCHEMA_FILE: &str = "policies/schema.cedarschema";

static POLICY_SET: LazyLock<PolicySet> = LazyLock::new(|| {
    let src = read_policy_src().unwrap_or_else(|e| {
        panic!("failed to read cedar policy file: {:?}", e);
    });
    PolicySet::from_str(&src).unwrap_or_else(|e| panic!("failed to parse cedar policy: {}", e))
});
static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    let src = read_schema_src().unwrap_or_else(|e| {
        panic!("failed to read cedar schema file: {:?}", e);
    });
    Schema::from_cedarschema_str(&src)
        .map(|(schema, _)| schema)
        .unwrap_or_else(|e| panic!("failed to parse cedar schema: {}", e))
});

pub fn init() -> Result<(), UseCaseError> {
    LazyLock::force(&POLICY_SET);
    LazyLock::force(&SCHEMA);
    Ok(())
}

pub enum PolicyEvaluation<F = Infallible> {
    Allow,
    Deny,
    Filter(F),
}

enum PartialPolicyEvaluation {
    Allow,
    Deny,
    Residual(PolicySet),
}

pub fn authorize_list_query(
    ctx: &UserContext,
    action: &str,
    resource_type: &str,
) -> Result<PolicyEvaluation, UseCaseError> {
    match authorize_partial_list_query(ctx, action, resource_type)? {
        PartialPolicyEvaluation::Allow => Ok(PolicyEvaluation::Allow),
        PartialPolicyEvaluation::Deny => Ok(PolicyEvaluation::Deny),
        PartialPolicyEvaluation::Residual(_) => {
            Err(unsupported_residual_filter_error(resource_type))
        }
    }
}

fn authorize_partial_list_query(
    ctx: &UserContext,
    action: &str,
    resource_type: &str,
) -> Result<PartialPolicyEvaluation, UseCaseError> {
    let principal_uid = entity_uid(ENTITY_TYPE_USER, &ctx.user_id().to_string())?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;
    let entities = Entities::from_entities([principal], Some(&SCHEMA)).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build cedar entities: {e}"))
    })?;
    let partial_entities = PartialEntities::from_concrete(entities, &SCHEMA).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build cedar partial entities: {e}"))
    })?;
    let request = PartialRequest::new(
        PartialEntityUid::from_concrete(principal_uid),
        entity_uid("Action", action)?,
        PartialEntityUid::new(entity_type_name(resource_type)?, None),
        Some(Context::empty()),
        &SCHEMA,
    )
    .map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build cedar partial request: {e}"))
    })?;

    let response = POLICY_SET
        .tpe(&request, &partial_entities, &SCHEMA)
        .map_err(|e| UseCaseError::AuthorizationError(format!("cedar TPE failed: {e}")))?;
    match response.decision() {
        Some(Decision::Allow) => Ok(PartialPolicyEvaluation::Allow),
        Some(Decision::Deny) => Ok(PartialPolicyEvaluation::Deny),
        None => {
            let residuals: Vec<_> = response.nontrivial_residual_policies().collect();
            if residuals.is_empty() {
                Ok(PartialPolicyEvaluation::Deny)
            } else {
                Ok(PartialPolicyEvaluation::Residual(residual_policy_set(
                    residuals,
                )?))
            }
        }
    }
}

pub fn authorize_book_query(
    ctx: &UserContext,
    action: &str,
) -> Result<PolicyEvaluation<book::ListFilter>, UseCaseError> {
    match authorize_partial_list_query(ctx, action, ENTITY_TYPE_BOOK)? {
        PartialPolicyEvaluation::Allow => Ok(PolicyEvaluation::Allow),
        PartialPolicyEvaluation::Deny => Ok(PolicyEvaluation::Deny),
        PartialPolicyEvaluation::Residual(residuals) => {
            if let Some(filter) = book_filter_from_residual_policies(ctx, &residuals) {
                return Ok(PolicyEvaluation::Filter(filter));
            }

            Err(unsupported_residual_filter_error(ENTITY_TYPE_BOOK))
        }
    }
}

fn unsupported_residual_filter_error(resource_type: &str) -> UseCaseError {
    UseCaseError::AuthorizationError(format!(
        "cedar residual policy for {resource_type} could not be converted to a list filter"
    ))
}

fn residual_policy_set(residuals: Vec<CedarPolicy>) -> Result<PolicySet, UseCaseError> {
    PolicySet::from_policies(residuals).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build residual policy set: {e}"))
    })
}

fn book_filter_from_residual_policies(
    ctx: &UserContext,
    policies: &PolicySet,
) -> Option<book::ListFilter> {
    // Translate only the Cedar subset we can prove equivalent to the DB filter.
    // Unsupported residuals return None so the caller can reject the query.
    policies
        .policies()
        .map(|policy| book_filter_from_residual_policy(ctx, policy))
        .try_fold(book::ListFilter::None, |acc, filter| {
            Some(book::ListFilter::or(acc, filter?))
        })
}

fn book_filter_from_residual_policy(
    ctx: &UserContext,
    policy: &CedarPolicy,
) -> Option<book::ListFilter> {
    let policy = policy.to_pst().ok()?;
    let body = policy.body();
    if body.effect != pst::Effect::Permit {
        return None;
    }

    body.clauses()
        .iter()
        .map(|clause| match clause {
            Clause::When(expr) => book_filter_from_expr(ctx, expr),
            Clause::Unless(_) => None,
        })
        .try_fold(book::ListFilter::All, |acc, filter| {
            Some(book::ListFilter::and(acc, filter?))
        })
}

fn book_filter_from_expr(ctx: &UserContext, expr: &Expr) -> Option<book::ListFilter> {
    match expr {
        Expr::Literal(Literal::Bool(true)) => Some(book::ListFilter::All),
        Expr::Literal(Literal::Bool(false)) => Some(book::ListFilter::None),
        Expr::GetAttr { expr, attr }
            if matches!(expr.as_ref(), Expr::Var(Var::Principal))
                && attr.as_str() == "is_admin" =>
        {
            Some(if ctx.is_admin() {
                book::ListFilter::All
            } else {
                book::ListFilter::None
            })
        }
        Expr::UnaryOp {
            op: UnaryOp::Not,
            expr,
        } => Some(book::ListFilter::negate(book_filter_from_expr(ctx, expr)?)),
        Expr::BinaryOp { op, left, right } => match op {
            BinaryOp::And => Some(book::ListFilter::and(
                book_filter_from_expr(ctx, left)?,
                book_filter_from_expr(ctx, right)?,
            )),
            BinaryOp::Or => Some(book::ListFilter::or(
                book_filter_from_expr(ctx, left)?,
                book_filter_from_expr(ctx, right)?,
            )),
            BinaryOp::Eq => book_owner_filter_from_eq(ctx, left, right)
                .or_else(|| book_owner_filter_from_eq(ctx, right, left)),
            _ => None,
        },
        _ => None,
    }
}

fn book_owner_filter_from_eq(
    ctx: &UserContext,
    left: &Expr,
    right: &Expr,
) -> Option<book::ListFilter> {
    if !is_resource_user_id(left) {
        return None;
    }
    Some(book::ListFilter::owned_by(user_id_value(ctx, right)?))
}

fn is_resource_user_id(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::GetAttr { expr, attr }
            if matches!(expr.as_ref(), Expr::Var(Var::Resource)) && attr.as_str() == "user_id"
    )
}

fn user_id_value(ctx: &UserContext, expr: &Expr) -> Option<uuid::Uuid> {
    match expr {
        Expr::Literal(Literal::String(value)) => uuid::Uuid::parse_str(value).ok(),
        Expr::GetAttr { expr, attr }
            if matches!(expr.as_ref(), Expr::Var(Var::Principal)) && attr.as_str() == "user_id" =>
        {
            Some(*ctx.user_id())
        }
        _ => None,
    }
}

fn authorize_resources_batch<T, ResourceUid, ResourceEntity>(
    ctx: &UserContext,
    action: &str,
    residual_policies: &PolicySet,
    resources: &[T],
    mut resource_uid: ResourceUid,
    mut resource_entity: ResourceEntity,
) -> Result<Vec<T>, UseCaseError>
where
    T: Clone,
    ResourceUid: FnMut(&T) -> Result<EntityUid, UseCaseError>,
    ResourceEntity: FnMut(&T) -> Result<Entity, UseCaseError>,
{
    let authorizer = Authorizer::new();
    let principal_uid = entity_uid(ENTITY_TYPE_USER, &ctx.user_id().to_string())?;
    let action_uid = entity_uid("Action", action)?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;

    let mut allowed = Vec::new();
    for item in resources {
        let resource_uid = resource_uid(item)?;
        let resource = resource_entity(item)?;
        let entities = Entities::from_entities([principal.clone(), resource], Some(&SCHEMA))
            .map_err(|e| {
                UseCaseError::AuthorizationError(format!("failed to build cedar entities: {e}"))
            })?;
        let request = Request::new(
            principal_uid.clone(),
            action_uid.clone(),
            resource_uid,
            Context::empty(),
            Some(&SCHEMA),
        )
        .map_err(|e| {
            UseCaseError::AuthorizationError(format!("failed to build cedar request: {e}"))
        })?;
        let decision = authorizer
            .is_authorized(&request, residual_policies, &entities)
            .decision();
        if decision == Decision::Allow {
            allowed.push(item.clone());
        }
    }

    Ok(allowed)
}

pub fn authorize_books_batch(
    ctx: &UserContext,
    action: &str,
    residual_policies: &PolicySet,
    books: &[book::Book],
) -> Result<Vec<book::Book>, UseCaseError> {
    authorize_resources_batch(
        ctx,
        action,
        residual_policies,
        books,
        |book| entity_uid(ENTITY_TYPE_BOOK, &book.pub_id().to_string()),
        |book| book_entity(&book.pub_id(), book.user_id()),
    )
}

pub fn authorize_publishers_batch(
    ctx: &UserContext,
    action: &str,
    residual_policies: &PolicySet,
    publishers: &[publisher::Publisher],
) -> Result<Vec<publisher::Publisher>, UseCaseError> {
    authorize_resources_batch(
        ctx,
        action,
        residual_policies,
        publishers,
        |publisher| entity_uid(ENTITY_TYPE_PUBLISHER, &publisher.pub_id().to_string()),
        |publisher| publisher_entity(&publisher.pub_id()),
    )
}

pub fn authorize_shops_batch(
    ctx: &UserContext,
    action: &str,
    residual_policies: &PolicySet,
    shops: &[shop::Shop],
) -> Result<Vec<shop::Shop>, UseCaseError> {
    authorize_resources_batch(
        ctx,
        action,
        residual_policies,
        shops,
        |shop| entity_uid(ENTITY_TYPE_SHOP, &shop.pub_id().to_string()),
        |shop| shop_entity(&shop.pub_id()),
    )
}

pub fn authorize_book_action(
    ctx: &UserContext,
    action: &str,
    book: &book::Book,
) -> Result<(), UseCaseError> {
    let resource_uid = entity_uid(ENTITY_TYPE_BOOK, &book.pub_id().to_string())?;
    let resource = book_entity(&book.pub_id(), book.user_id())?;
    authorize_action_with_resource(ctx, action, resource_uid, resource)
}

pub fn authorize_publisher_action(
    ctx: &UserContext,
    action: &str,
    publisher: &publisher::Publisher,
) -> Result<(), UseCaseError> {
    let resource_uid = entity_uid(ENTITY_TYPE_PUBLISHER, &publisher.pub_id().to_string())?;
    let resource = publisher_entity(&publisher.pub_id())?;
    authorize_action_with_resource(ctx, action, resource_uid, resource)
}

pub fn authorize_shop_action(
    ctx: &UserContext,
    action: &str,
    shop: &shop::Shop,
) -> Result<(), UseCaseError> {
    let resource_uid = entity_uid(ENTITY_TYPE_SHOP, &shop.pub_id().to_string())?;
    let resource = shop_entity(&shop.pub_id())?;
    authorize_action_with_resource(ctx, action, resource_uid, resource)
}

fn authorize_action_with_resource(
    ctx: &UserContext,
    action: &str,
    resource_uid: EntityUid,
    resource: Entity,
) -> Result<(), UseCaseError> {
    let authorizer = Authorizer::new();
    let principal_uid = entity_uid(ENTITY_TYPE_USER, &ctx.user_id().to_string())?;
    let action_uid = entity_uid("Action", action)?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;
    let entities = Entities::from_entities([principal, resource], Some(&SCHEMA)).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build cedar entities: {e}"))
    })?;
    let request = Request::new(
        principal_uid,
        action_uid,
        resource_uid,
        Context::empty(),
        Some(&SCHEMA),
    )
    .map_err(|e| UseCaseError::AuthorizationError(format!("failed to build cedar request: {e}")))?;

    let decision = authorizer
        .is_authorized(&request, &POLICY_SET, &entities)
        .decision();
    if decision == Decision::Allow {
        Ok(())
    } else {
        Err(UseCaseError::Forbidden("not authorized".to_string()))
    }
}

fn read_policy_src() -> Result<String, UseCaseError> {
    let path = policy_path();
    fs::read_to_string(&path).map_err(|e| {
        let context = format!("failed to read cedar policy file: {}", path.display());
        UseCaseError::AuthorizationError(format!("{context}: {e}"))
    })
}

fn read_schema_src() -> Result<String, UseCaseError> {
    let path = schema_path();
    fs::read_to_string(&path).map_err(|e| {
        let context = format!("failed to read cedar schema file: {}", path.display());
        UseCaseError::AuthorizationError(format!("{context}: {e}"))
    })
}

fn policy_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(POLICY_FILE)
}

fn schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCHEMA_FILE)
}

fn user_entity(user_id: &uuid::Uuid, is_admin: bool) -> Result<Entity, UseCaseError> {
    let uid = entity_uid(ENTITY_TYPE_USER, &user_id.to_string())?;
    let attrs = HashMap::from([
        ("user_id".to_string(), string_expr(&user_id.to_string())?),
        ("is_admin".to_string(), bool_expr(is_admin)?),
    ]);
    Entity::new(uid, attrs, HashSet::new()).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build principal entity: {e}"))
    })
}

fn book_entity(book_id: &uuid::Uuid, user_id: &uuid::Uuid) -> Result<Entity, UseCaseError> {
    let uid = entity_uid(ENTITY_TYPE_BOOK, &book_id.to_string())?;
    let attrs = HashMap::from([("user_id".to_string(), string_expr(&user_id.to_string())?)]);
    Entity::new(uid, attrs, HashSet::new()).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build resource entity: {e}"))
    })
}

fn publisher_entity(publisher_id: &uuid::Uuid) -> Result<Entity, UseCaseError> {
    let uid = entity_uid(ENTITY_TYPE_PUBLISHER, &publisher_id.to_string())?;
    Entity::new(uid, HashMap::new(), HashSet::new()).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build resource entity: {e}"))
    })
}

fn shop_entity(shop_id: &uuid::Uuid) -> Result<Entity, UseCaseError> {
    let uid = entity_uid(ENTITY_TYPE_SHOP, &shop_id.to_string())?;
    Entity::new(uid, HashMap::new(), HashSet::new()).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build resource entity: {e}"))
    })
}

fn string_expr(value: &str) -> Result<RestrictedExpression, UseCaseError> {
    RestrictedExpression::from_str(&format!("\"{}\"", value)).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build cedar string expression: {e}"))
    })
}

fn bool_expr(value: bool) -> Result<RestrictedExpression, UseCaseError> {
    RestrictedExpression::from_str(if value { "true" } else { "false" }).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to build cedar bool expression: {e}"))
    })
}

fn entity_uid(entity_type: &str, id: &str) -> Result<EntityUid, UseCaseError> {
    let type_name = entity_type_name(entity_type)?;
    let entity_id = EntityId::from_str(id).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to parse cedar entity id: {e}"))
    })?;
    Ok(EntityUid::from_type_name_and_id(type_name, entity_id))
}

fn entity_type_name(name: &str) -> Result<EntityTypeName, UseCaseError> {
    EntityTypeName::from_str(name).map_err(|e| {
        UseCaseError::AuthorizationError(format!("failed to parse cedar entity type name: {e}"))
    })
}
