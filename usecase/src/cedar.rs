use crate::{UserContext, error::UseCaseError};
use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityId, EntityTypeName, EntityUid,
    PolicySet, Request, RestrictedExpression, Schema, TestEntityLoader,
};
use cedar_schema_macros::cedar_schema_consts;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    str::FromStr,
    sync::LazyLock,
};

// 黒魔術: cedarschemaからconst作成
cedar_schema_consts!("policies/schema.cedarschema");

const POLICY_FILE: &str = "policies/book.cedar";
const SCHEMA_FILE: &str = "policies/schema.cedarschema";

static POLICY_SET: LazyLock<Result<PolicySet, UseCaseError>> = LazyLock::new(|| {
    let src = read_policy_src()?;
    PolicySet::from_str(&src).map_err(|e| cedar_error("failed to parse cedar policy", e))
});
static SCHEMA: LazyLock<Result<Schema, UseCaseError>> = LazyLock::new(|| {
    let src = read_schema_src()?;
    Schema::from_cedarschema_str(&src)
        .map(|(schema, _)| schema)
        .map_err(|e| cedar_error("failed to parse cedar schema", e))
});

pub enum PartialDecision {
    Allow,
    Deny,
    Residual(Box<PolicySet>),
}

pub fn init() -> Result<(), UseCaseError> {
    policy_set()?;
    schema()?;
    Ok(())
}

pub fn partial_authorize_book_list(ctx: &UserContext) -> Result<PartialDecision, UseCaseError> {
    partial_authorize(ctx, ACTION_LIST_BOOKS, ENTITY_TYPE_BOOK)
}

pub fn authorize_book_list_batch(
    ctx: &UserContext,
    books: &[book::Book],
) -> Result<Vec<book::Book>, UseCaseError> {
    let policies = policy_set()?;
    authorize_books_batch(ctx, ACTION_LIST_BOOKS, policies, books)
}

pub fn authorize_book_get(ctx: &UserContext, book: &book::Book) -> Result<(), UseCaseError> {
    authorize_book_action(ctx, ACTION_GET_BOOK, book)
}

pub fn authorize_book_create(ctx: &UserContext, book: &book::Book) -> Result<(), UseCaseError> {
    authorize_book_action(ctx, ACTION_CREATE_BOOK, book)
}

pub fn authorize_book_update(ctx: &UserContext, book: &book::Book) -> Result<(), UseCaseError> {
    authorize_book_action(ctx, ACTION_UPDATE_BOOK, book)
}

pub fn authorize_book_delete(ctx: &UserContext, book: &book::Book) -> Result<(), UseCaseError> {
    authorize_book_action(ctx, ACTION_DELETE_BOOK, book)
}

pub fn authorize_book_change_applied_at(
    ctx: &UserContext,
    book: &book::Book,
) -> Result<(), UseCaseError> {
    authorize_book_action(ctx, ACTION_CHANGE_BOOK_APPLIED_AT, book)
}

pub fn partial_authorize_publisher_list(
    ctx: &UserContext,
) -> Result<PartialDecision, UseCaseError> {
    partial_authorize(ctx, ACTION_LIST_PUBLISHERS, ENTITY_TYPE_PUBLISHER)
}

pub fn authorize_publisher_list_batch(
    ctx: &UserContext,
    residual_policies: &PolicySet,
    publishers: &[publisher::Publisher],
) -> Result<Vec<publisher::Publisher>, UseCaseError> {
    authorize_publishers_batch(ctx, ACTION_LIST_PUBLISHERS, residual_policies, publishers)
}

pub fn authorize_publisher_get(
    ctx: &UserContext,
    publisher: &publisher::Publisher,
) -> Result<(), UseCaseError> {
    authorize_publisher_action(ctx, ACTION_GET_PUBLISHER, publisher)
}

pub fn authorize_publisher_create(
    ctx: &UserContext,
    publisher: &publisher::Publisher,
) -> Result<(), UseCaseError> {
    authorize_publisher_action(ctx, ACTION_CREATE_PUBLISHER, publisher)
}

pub fn authorize_publisher_update(
    ctx: &UserContext,
    publisher: &publisher::Publisher,
) -> Result<(), UseCaseError> {
    authorize_publisher_action(ctx, ACTION_UPDATE_PUBLISHER, publisher)
}

pub fn authorize_publisher_delete(
    ctx: &UserContext,
    publisher: &publisher::Publisher,
) -> Result<(), UseCaseError> {
    authorize_publisher_action(ctx, ACTION_DELETE_PUBLISHER, publisher)
}

pub fn partial_authorize_shop_list(ctx: &UserContext) -> Result<PartialDecision, UseCaseError> {
    partial_authorize(ctx, ACTION_LIST_SHOPS, ENTITY_TYPE_SHOP)
}

pub fn authorize_shop_list_batch(
    ctx: &UserContext,
    residual_policies: &PolicySet,
    shops: &[shop::Shop],
) -> Result<Vec<shop::Shop>, UseCaseError> {
    authorize_shops_batch(ctx, ACTION_LIST_SHOPS, residual_policies, shops)
}

pub fn authorize_shop_get(ctx: &UserContext, shop: &shop::Shop) -> Result<(), UseCaseError> {
    authorize_shop_action(ctx, ACTION_GET_SHOP, shop)
}

pub fn authorize_shop_create(ctx: &UserContext, shop: &shop::Shop) -> Result<(), UseCaseError> {
    authorize_shop_action(ctx, ACTION_CREATE_SHOP, shop)
}

pub fn authorize_shop_update(ctx: &UserContext, shop: &shop::Shop) -> Result<(), UseCaseError> {
    authorize_shop_action(ctx, ACTION_UPDATE_SHOP, shop)
}

pub fn authorize_shop_delete(ctx: &UserContext, shop: &shop::Shop) -> Result<(), UseCaseError> {
    authorize_shop_action(ctx, ACTION_DELETE_SHOP, shop)
}

pub fn authorize_dashboard_get_annual_summary(ctx: &UserContext) -> Result<(), UseCaseError> {
    authorize_dashboard_action(ctx, ACTION_GET_ANNUAL_SUMMARY)
}

fn partial_authorize(
    ctx: &UserContext,
    action: &str,
    resource_type: &str,
) -> Result<PartialDecision, UseCaseError> {
    let policies = policy_set()?;
    let authorizer = Authorizer::new();
    let principal_uid = user_uid(ctx.user_id())?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;
    let entities = Entities::from_entities([principal], None)
        .map_err(|e| cedar_error("failed to build cedar entities", e))?;

    let request = Request::builder()
        .principal(principal_uid)
        .action(action_uid(action)?)
        .unknown_resource_with_type(entity_type_name(resource_type)?)
        .context(Context::empty())
        .build();

    let response = authorizer.is_authorized_partial(&request, policies, &entities);
    match response.decision() {
        Some(Decision::Allow) => Ok(PartialDecision::Allow),
        Some(Decision::Deny) => Ok(PartialDecision::Deny),
        None => {
            let residuals: Vec<_> = response.nontrivial_residuals().collect();
            if residuals.is_empty() {
                Ok(PartialDecision::Deny)
            } else {
                let residual_set = PolicySet::from_policies(residuals)
                    .map_err(|e| cedar_error("failed to build residual policy set", e))?;
                Ok(PartialDecision::Residual(Box::new(residual_set)))
            }
        }
    }
}

fn authorize_books_batch(
    ctx: &UserContext,
    action: &str,
    policies: &PolicySet,
    books: &[book::Book],
) -> Result<Vec<book::Book>, UseCaseError> {
    let schema = schema()?;
    let principal_uid = user_uid(ctx.user_id())?;
    let action_uid = action_uid(action)?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;

    let mut allowed = Vec::new();
    for book in books {
        let resource_uid = book_uid(&book.pub_id())?;
        let resource = book_entity(&book.pub_id(), book.user_id())?;
        let entities = Entities::from_entities([principal.clone(), resource], Some(schema))
            .map_err(|e| cedar_error("failed to build cedar entities", e))?;
        let request = Request::new(
            principal_uid.clone(),
            action_uid.clone(),
            resource_uid,
            Context::empty(),
            Some(schema),
        )
        .map_err(|e| cedar_error("failed to build cedar request", e))?;
        let mut loader = TestEntityLoader::new(&entities);
        let decision = policies
            .is_authorized_batched(&request, schema, &mut loader, u32::MAX)
            .map_err(|e| cedar_error("cedar batch evaluation failed", e))?;
        if decision == Decision::Allow {
            allowed.push(book.clone());
        }
    }

    Ok(allowed)
}

fn authorize_publishers_batch(
    ctx: &UserContext,
    action: &str,
    residual_policies: &PolicySet,
    publishers: &[publisher::Publisher],
) -> Result<Vec<publisher::Publisher>, UseCaseError> {
    let schema = schema()?;
    let authorizer = Authorizer::new();
    let principal_uid = user_uid(ctx.user_id())?;
    let action_uid = action_uid(action)?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;

    let mut allowed = Vec::new();
    for publisher in publishers {
        let resource_uid = publisher_uid(&publisher.pub_id())?;
        let resource = publisher_entity(&publisher.pub_id())?;
        let entities = Entities::from_entities([principal.clone(), resource], Some(schema))
            .map_err(|e| cedar_error("failed to build cedar entities", e))?;
        let request = Request::new(
            principal_uid.clone(),
            action_uid.clone(),
            resource_uid,
            Context::empty(),
            Some(schema),
        )
        .map_err(|e| cedar_error("failed to build cedar request", e))?;
        let decision = authorizer
            .is_authorized(&request, residual_policies, &entities)
            .decision();
        if decision == Decision::Allow {
            allowed.push(publisher.clone());
        }
    }

    Ok(allowed)
}

fn authorize_shops_batch(
    ctx: &UserContext,
    action: &str,
    residual_policies: &PolicySet,
    shops: &[shop::Shop],
) -> Result<Vec<shop::Shop>, UseCaseError> {
    let schema = schema()?;
    let authorizer = Authorizer::new();
    let principal_uid = user_uid(ctx.user_id())?;
    let action_uid = action_uid(action)?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;

    let mut allowed = Vec::new();
    for shop in shops {
        let resource_uid = shop_uid(&shop.pub_id())?;
        let resource = shop_entity(&shop.pub_id())?;
        let entities = Entities::from_entities([principal.clone(), resource], Some(schema))
            .map_err(|e| cedar_error("failed to build cedar entities", e))?;
        let request = Request::new(
            principal_uid.clone(),
            action_uid.clone(),
            resource_uid,
            Context::empty(),
            Some(schema),
        )
        .map_err(|e| cedar_error("failed to build cedar request", e))?;
        let decision = authorizer
            .is_authorized(&request, residual_policies, &entities)
            .decision();
        if decision == Decision::Allow {
            allowed.push(shop.clone());
        }
    }

    Ok(allowed)
}

fn authorize_book_action(
    ctx: &UserContext,
    action: &str,
    book: &book::Book,
) -> Result<(), UseCaseError> {
    let resource_uid = book_uid(&book.pub_id())?;
    let resource = book_entity(&book.pub_id(), book.user_id())?;
    authorize_action_with_resource(ctx, action, resource_uid, resource)
}

fn authorize_publisher_action(
    ctx: &UserContext,
    action: &str,
    publisher: &publisher::Publisher,
) -> Result<(), UseCaseError> {
    let resource_uid = publisher_uid(&publisher.pub_id())?;
    let resource = publisher_entity(&publisher.pub_id())?;
    authorize_action_with_resource(ctx, action, resource_uid, resource)
}

fn authorize_shop_action(
    ctx: &UserContext,
    action: &str,
    shop: &shop::Shop,
) -> Result<(), UseCaseError> {
    let resource_uid = shop_uid(&shop.pub_id())?;
    let resource = shop_entity(&shop.pub_id())?;
    authorize_action_with_resource(ctx, action, resource_uid, resource)
}

fn authorize_dashboard_action(ctx: &UserContext, action: &str) -> Result<(), UseCaseError> {
    let resource_uid = dashboard_uid()?;
    let resource = dashboard_entity()?;
    authorize_action_with_resource(ctx, action, resource_uid, resource)
}

fn authorize_action_with_resource(
    ctx: &UserContext,
    action: &str,
    resource_uid: EntityUid,
    resource: Entity,
) -> Result<(), UseCaseError> {
    let policies = policy_set()?;
    let schema = schema()?;
    let authorizer = Authorizer::new();
    let principal_uid = user_uid(ctx.user_id())?;
    let action_uid = action_uid(action)?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;
    let entities = Entities::from_entities([principal, resource], Some(schema))
        .map_err(|e| cedar_error("failed to build cedar entities", e))?;
    let request = Request::new(
        principal_uid,
        action_uid,
        resource_uid,
        Context::empty(),
        Some(schema),
    )
    .map_err(|e| cedar_error("failed to build cedar request", e))?;

    let decision = authorizer
        .is_authorized(&request, policies, &entities)
        .decision();
    if decision == Decision::Allow {
        Ok(())
    } else {
        Err(UseCaseError::Forbidden("not authorized".to_string()))
    }
}

fn policy_set() -> Result<&'static PolicySet, UseCaseError> {
    POLICY_SET
        .as_ref()
        .map_err(|e| cedar_error("failed to load cedar policy", e))
}

fn schema() -> Result<&'static Schema, UseCaseError> {
    SCHEMA
        .as_ref()
        .map_err(|e| cedar_error("failed to load cedar schema", e))
}

fn read_policy_src() -> Result<String, UseCaseError> {
    let path = policy_path();
    fs::read_to_string(&path).map_err(|e| {
        let context = format!("failed to read cedar policy file: {}", path.display());
        cedar_error(&context, e)
    })
}

fn read_schema_src() -> Result<String, UseCaseError> {
    let path = schema_path();
    fs::read_to_string(&path).map_err(|e| {
        let context = format!("failed to read cedar schema file: {}", path.display());
        cedar_error(&context, e)
    })
}

fn policy_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(POLICY_FILE)
}

fn schema_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SCHEMA_FILE)
}

fn user_uid(user_id: &uuid::Uuid) -> Result<EntityUid, UseCaseError> {
    entity_uid(ENTITY_TYPE_USER, &user_id.to_string())
}

fn book_uid(book_id: &uuid::Uuid) -> Result<EntityUid, UseCaseError> {
    entity_uid(ENTITY_TYPE_BOOK, &book_id.to_string())
}

fn publisher_uid(publisher_id: &uuid::Uuid) -> Result<EntityUid, UseCaseError> {
    entity_uid(ENTITY_TYPE_PUBLISHER, &publisher_id.to_string())
}

fn shop_uid(shop_id: &uuid::Uuid) -> Result<EntityUid, UseCaseError> {
    entity_uid(ENTITY_TYPE_SHOP, &shop_id.to_string())
}

fn dashboard_uid() -> Result<EntityUid, UseCaseError> {
    entity_uid(ENTITY_TYPE_DASHBOARD, "default")
}

fn action_uid(action: &str) -> Result<EntityUid, UseCaseError> {
    entity_uid("Action", action)
}

fn user_entity(user_id: &uuid::Uuid, is_admin: bool) -> Result<Entity, UseCaseError> {
    let uid = user_uid(user_id)?;
    let attrs = HashMap::from([
        ("user_id".to_string(), string_expr(&user_id.to_string())?),
        ("is_admin".to_string(), bool_expr(is_admin)?),
    ]);
    Entity::new(uid, attrs, HashSet::new())
        .map_err(|e| cedar_error("failed to build principal entity", e))
}

fn book_entity(book_id: &uuid::Uuid, user_id: &uuid::Uuid) -> Result<Entity, UseCaseError> {
    let uid = book_uid(book_id)?;
    let attrs = HashMap::from([("user_id".to_string(), string_expr(&user_id.to_string())?)]);
    Entity::new(uid, attrs, HashSet::new())
        .map_err(|e| cedar_error("failed to build resource entity", e))
}

fn publisher_entity(publisher_id: &uuid::Uuid) -> Result<Entity, UseCaseError> {
    let uid = publisher_uid(publisher_id)?;
    Entity::new(uid, HashMap::new(), HashSet::new())
        .map_err(|e| cedar_error("failed to build resource entity", e))
}

fn shop_entity(shop_id: &uuid::Uuid) -> Result<Entity, UseCaseError> {
    let uid = shop_uid(shop_id)?;
    Entity::new(uid, HashMap::new(), HashSet::new())
        .map_err(|e| cedar_error("failed to build resource entity", e))
}

fn dashboard_entity() -> Result<Entity, UseCaseError> {
    let uid = dashboard_uid()?;
    Entity::new(uid, HashMap::new(), HashSet::new())
        .map_err(|e| cedar_error("failed to build resource entity", e))
}

fn string_expr(value: &str) -> Result<RestrictedExpression, UseCaseError> {
    RestrictedExpression::from_str(&format!("\"{}\"", value))
        .map_err(|e| cedar_error("failed to build cedar string expression", e))
}

fn bool_expr(value: bool) -> Result<RestrictedExpression, UseCaseError> {
    RestrictedExpression::from_str(if value { "true" } else { "false" })
        .map_err(|e| cedar_error("failed to build cedar bool expression", e))
}

fn entity_uid(entity_type: &str, id: &str) -> Result<EntityUid, UseCaseError> {
    let type_name = entity_type_name(entity_type)?;
    let entity_id =
        EntityId::from_str(id).map_err(|e| cedar_error("failed to parse cedar entity id", e))?;
    Ok(EntityUid::from_type_name_and_id(type_name, entity_id))
}

fn entity_type_name(name: &str) -> Result<EntityTypeName, UseCaseError> {
    EntityTypeName::from_str(name)
        .map_err(|e| cedar_error("failed to parse cedar entity type name", e))
}

fn cedar_error(context: &str, err: impl std::fmt::Display) -> UseCaseError {
    eprintln!("{}: {}", context, err);
    UseCaseError::InternalServerError
}
