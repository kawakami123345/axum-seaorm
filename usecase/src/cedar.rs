use crate::{UserContext, error::UseCaseError};
use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityId, EntityTypeName, EntityUid,
    PolicySet, Request, RestrictedExpression, Schema, TestEntityLoader,
};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    str::FromStr,
    sync::LazyLock,
};

const ENTITY_TYPE_USER: &str = "User";
const ENTITY_TYPE_BOOK: &str = "Book";
const ACTION_GET_BOOK: &str = "GetBook";

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

pub fn partial_authorize_books(ctx: &UserContext) -> Result<PartialDecision, UseCaseError> {
    let policies = policy_set()?;
    let authorizer = Authorizer::new();
    let principal_uid = user_uid(ctx.user_id())?;
    let principal = user_entity(ctx.user_id(), ctx.is_admin())?;
    let entities = Entities::from_entities([principal], None)
        .map_err(|e| cedar_error("failed to build cedar entities", e))?;

    let request = Request::builder()
        .principal(principal_uid)
        .action(action_uid()?)
        .unknown_resource_with_type(entity_type_name(ENTITY_TYPE_BOOK)?)
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

pub fn authorize_books_batch(
    ctx: &UserContext,
    residual_policies: &PolicySet,
    books: &[book::Book],
) -> Result<Vec<book::Book>, UseCaseError> {
    let schema = schema()?;
    let principal_uid = user_uid(ctx.user_id())?;
    let action_uid = action_uid()?;
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
        let decision = residual_policies
            .is_authorized_batched(&request, schema, &mut loader, u32::MAX)
            .map_err(|e| cedar_error("cedar batch evaluation failed", e))?;
        if decision == Decision::Allow {
            allowed.push(book.clone());
        }
    }

    Ok(allowed)
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

fn action_uid() -> Result<EntityUid, UseCaseError> {
    entity_uid("Action", ACTION_GET_BOOK)
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
