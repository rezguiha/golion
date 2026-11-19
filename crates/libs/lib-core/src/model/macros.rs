/// Generates default common default crud operations.
/// In case of specific implementation of create or update.
/// Omit them from the macro and implement your own.
#[macro_export]
macro_rules! generate_common_crud_controllers {
    (
        ModController:$struct_name:ident,
        Entity:$entity:ty,
        $(ForCreate:$for_create:ty,)?
        $(ForUpdate:$for_update:ty,)?
    ) => {
        impl $struct_name {
            $(pub async fn create(
                ctx: &Ctx,
                mm: &ModelManager,
                entity_c: $for_create,
            ) -> $crate::Result<i64> {
                base::create::<Self, _>(ctx, mm, entity_c).await
            })?
            pub async fn get(
                ctx: &Ctx,
                mm: &ModelManager,
                id: i64,
            ) -> $crate::Result<$entity> {
                base::get::<Self, _>(ctx, mm, id).await
            }
            pub async fn list(
                ctx: &Ctx,
                mm: &ModelManager,
            ) -> $crate::Result<Vec<$entity>> {
                base::list::<Self, _>(ctx, mm).await
            }
            pub async fn delete(
                ctx: &Ctx,
                mm: &ModelManager,
                id: i64,
            ) -> $crate::Result<()> {
                base::delete::<Self>(ctx, mm, id).await
            }
            $(
            pub async fn update(
                ctx: &Ctx,
                mm: &ModelManager,
                id: i64,
                entity_u: $for_update,
            ) -> $crate::Result<()> {
                base::update::<Self, _>(ctx, mm, id, entity_u).await
            })?
        }
    };
}
