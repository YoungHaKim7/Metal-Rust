#![forbid(unsafe_code)]

use metal_rust::Error;
use metal_rust::metal::{
    Buffer, PurgeableState, Resource, ResourceID, ResourceOwnerIdentity, ResourceViewPool, Texture,
    TextureDescriptor, TextureViewDescriptor, TextureViewPool,
};

#[allow(dead_code)]
fn safe_resource_tail_surface(
    resource: &Resource,
    source: &ResourceViewPool,
    destination: &ResourceViewPool,
    owner: &ResourceOwnerIdentity,
) -> Result<(bool, PurgeableState, ResourceID, ResourceID), Error> {
    let aliasable = resource.is_aliasable()?;
    resource.set_owner(owner)?;
    let purgeable = resource.set_purgeable_state(PurgeableState::PurgeableStateKeepCurrent)?;
    let base = destination.base_resource_id()?;
    let copied = destination.copy_resource_views_from_pool(source, 0..1, 0)?;
    Ok((aliasable, purgeable, base, copied))
}

#[allow(dead_code)]
fn safe_texture_view_pool_surface(
    pool: &TextureViewPool,
    texture: &Texture,
    buffer: &Buffer,
    view_descriptor: &TextureViewDescriptor,
    texture_descriptor: &TextureDescriptor,
) -> Result<(ResourceID, ResourceID, ResourceID), Error> {
    let default_view = pool.set_texture_view(texture, 0)?;
    let described_view = pool.set_texture_view_with_descriptor(texture, view_descriptor, 0)?;
    let buffer_view = pool.set_texture_view_from_buffer(buffer, texture_descriptor, 0, 256, 0)?;
    Ok((default_view, described_view, buffer_view))
}

#[test]
fn current_process_identity_is_safe_to_construct() {
    let _identity = ResourceOwnerIdentity::current_process().expect("current task identity token");
}
