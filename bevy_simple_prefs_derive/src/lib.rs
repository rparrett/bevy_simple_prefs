//! Derive macro for `bevy_simple_prefs`.

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro for `bevy_simple_prefs`.
#[proc_macro_derive(Prefs)]
pub fn prefs_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the struct name
    let name = &input.ident;

    // Generate the code
    let expanded = match input.data {
        Data::Struct(ref data_struct) => {
            let mut field_bindings = Vec::new();
            let mut field_checks = Vec::new();
            let mut fields = Vec::new();
            let mut field_assignments = Vec::new();
            let mut field_inits = Vec::new();
            let mut field_inserts = Vec::new();

            // Iterate over the fields of the struct
            match &data_struct.fields {
                Fields::Named(ref fields_named) => {
                    for field in &fields_named.named {
                        let field_name = &field.ident;
                        let field_type = &field.ty;

                        field_bindings.push(quote! {
                            let #field_name = world.get_resource_ref::<#field_type>().unwrap();
                        });
                        field_checks.push(quote! {
                            !#field_name.is_changed()
                        });
                        fields.push(quote! {
                            #field_name: #field_type
                        });
                        field_assignments.push(quote! {
                            #field_name: #field_name.clone()
                        });
                        field_inits.push(quote! {
                            app.init_resource::<#field_type>();
                        });
                        field_inserts.push(quote! {
                            world.insert_resource(val.#field_name);
                        });
                    }
                }
                _ => {
                    unimplemented!("Prefs can only be derived for structs with named fields")
                }
            }

            quote! {
                impl Prefs for #name {
                    fn save(world: &mut World) {
                        #(#field_bindings)*

                        if #(#field_checks)&&* {
                            return;
                        }

                        // Prevent saving from happening on the initial change detection after
                        // inserting the resources on load.
                        let status = world.get_resource_ref::<::bevy_simple_prefs::PrefsStatus<#name>>().unwrap();
                        if status.is_changed() {
                            return;
                        }

                        ::bevy::log::debug!("bevy_simple_prefs initiating save");

                        let to_save = #name {
                            #(#field_assignments,)*
                        };

                        let settings = world.resource::<::bevy_simple_prefs::PrefsSettings<#name>>();
                        let path = settings.path.clone();
                        let filename = settings.filename.clone();

                        ::bevy::tasks::IoTaskPool::get()
                            .spawn(async move {
                                ::bevy::log::debug!("bevy_simple_prefs saving");

                                let Ok(serialized_value) = ::bevy_simple_prefs::serialize(&to_save) else {
                                    bevy::log::error!("Failed to serialize prefs.");
                                    return;
                                };

                                ::bevy_simple_prefs::save_str(&path, &filename, &serialized_value);
                            }).detach();
                    }

                    #[cfg(not(target_arch = "wasm32"))]
                    fn load(world: &mut World) {
                        ::bevy::log::debug!("bevy_simple_prefs initiating load task");

                        let settings = world.resource::<::bevy_simple_prefs::PrefsSettings<#name>>();
                        let path = settings.path.clone();
                        let filename = settings.filename.clone();

                        let entity = world.spawn_empty().id();

                        let task = ::bevy::tasks::IoTaskPool::get().spawn(async move {
                            ::bevy::log::debug!("bevy_simple_prefs loading");

                            let val = (|| {
                                let Some(serialized_value) = ::bevy_simple_prefs::load_str(&path, &filename) else {
                                    return #name::default();
                                };

                                match ::bevy_simple_prefs::deserialize(&serialized_value) {
                                    Ok(v) => v,
                                    Err(e) => {
                                        ::bevy::log::error!("Failed to deserialize prefs: {}", e);
                                        return #name::default();
                                    }
                                }
                            })();

                            let mut command_queue = ::bevy::ecs::world::CommandQueue::default();
                            command_queue.push(move |world: &mut World| {
                                #(#field_inserts;)*;
                                world.resource_mut::<::bevy_simple_prefs::PrefsStatus<#name>>().loaded = true;
                                world.despawn(entity);
                            });

                            command_queue
                        });

                        world.entity_mut(entity).insert(::bevy_simple_prefs::LoadPrefsTask(task));
                    }

                    // There's no task pool and no multi-threading on wasm, so just load everything,
                    // toss it into the world, and update `PrefsStatus`.
                    #[cfg(target_arch = "wasm32")]
                    fn load(world: &mut World) {
                        ::bevy::log::debug!("bevy_simple_prefs loading");

                        let settings = world.resource::<::bevy_simple_prefs::PrefsSettings<#name>>();

                        let val = (|| {
                            let Some(serialized_value) = ::bevy_simple_prefs::load_str(&settings.path, &settings.filename) else {
                                return #name::default();
                            };

                            match ::bevy_simple_prefs::deserialize(&serialized_value) {
                                Ok(v) => v,
                                Err(e) => {
                                    ::bevy::log::error!("bevy_simple_prefs failed to deserialize prefs: {}", e);
                                    return #name::default();
                                }
                            }
                        })();

                        #(#field_inserts;)*;

                        world.resource_mut::<::bevy_simple_prefs::PrefsStatus<#name>>().loaded = true;
                    }

                    fn init(app: &mut App) {
                        #(#field_inits;)*
                    }
                }
            }
        }
        _ => unimplemented!("Prefs can only be derived for structs"),
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
