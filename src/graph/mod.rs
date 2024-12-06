use bevy::{
    prelude::*, 
	ecs::component::StorageType,
    render::mesh::Mesh2d
};

use crate::{
    colors::{HIGHLIGHT_COLOR, PRIMARY_COLOR}, shaders::PulsingMaterial
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GraphSystemsActive {
    #[default]
    False,
    True
}

pub struct GraphPlugin;
impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GraphSystemsActive>()
            .add_systems(Update, 
                activate_systems
            );
    }
}

fn activate_systems(
    mut graph_state: ResMut<NextState<GraphSystemsActive>>,
    graph_query: Query<&Graph>
) {
    graph_state.set(if graph_query.is_empty() {
        GraphSystemsActive::False
    } else {
        GraphSystemsActive::True
    });
}

#[derive(Component)]
pub struct GraphNode;

#[derive(Clone)]
pub struct Graph {
    inter_layer_distance : f32,
    num_nodes_per_layer : Vec<i32>,
    inter_node_distance : f32,
    node_outer_radius : f32,
    node_border_thickness : f32,
}

impl Graph {
    pub fn new(
        inter_layer_distance : f32,
        num_nodes_per_layer : Vec<i32>,
        inter_node_distance : f32,
        node_outer_radius : f32,
        node_border_thickness : f32
    ) -> Self {
        Graph {
            inter_layer_distance,
            num_nodes_per_layer,
            inter_node_distance,
            node_outer_radius,
            node_border_thickness
        }
    }

    // Function to compute x positions for a layer
    fn compute_x_positions(
        num_nodes: i32, 
        base_spacing: f32
    ) -> Vec<f32> {

        let middle_index = (num_nodes - 1) as f32 / 2.0;
        (0..num_nodes).map(|i| {
            let i = i as f32;
            (i - middle_index) * base_spacing
        }).collect()
    }

    // Refactor `spawn_graph_layer` accordingly
    fn spawn_layer(
        parent: &mut ChildBuilder<'_>,
        circle_mesh_handle: &Handle<Mesh>,
        annulus_mesh_handle: &Handle<Mesh>,
        node_material_slice: &[Handle<PulsingMaterial>],
        outline_material: &Handle<ColorMaterial>,
        transform: &Transform,
        x_positions: &[f32],
    ) {
        assert_eq!(
            x_positions.len(),
            node_material_slice.len(),
            "The number of positions and materials must be the same"
        );
    
        for (&x_position, material_handle) in x_positions.iter().zip(node_material_slice.iter()) {
            let node_transform = Transform::from_translation(Vec3::new(
                x_position + transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ));
    
            parent.spawn((
                Mesh2d(annulus_mesh_handle.clone()),
                MeshMaterial2d(outline_material.clone()),
                node_transform.clone()
            ));

            parent.spawn((
                GraphNode,
                Mesh2d(circle_mesh_handle.clone()),
                MeshMaterial2d(material_handle.clone()),
                node_transform.clone()
            ));
        }
    }

    fn spawn(
        parent: &mut ChildBuilder<'_>,
        circle_mesh_handle: Handle<Mesh>,
        annulus_mesh_handle: Handle<Mesh>,
        node_material_vector: Vec<Handle<PulsingMaterial>>,
        outline_material: Handle<ColorMaterial>,
        graph: Graph,
    ) {
        let base_spacing = graph.node_outer_radius * 2.0 + graph.inter_node_distance;
        let center_offset = ((graph.num_nodes_per_layer.len() - 1) as f32) / 2.0 * graph.inter_layer_distance;
    
        let mut material_index = 0;
    
        for (layer_index, &num_nodes) in graph.num_nodes_per_layer.iter().enumerate() {
            let y_position = layer_index as f32 * graph.inter_layer_distance - center_offset;
    
            // Compute x positions
            let x_positions = Graph::compute_x_positions(num_nodes, base_spacing);
    
            // Calculate the slice of materials for this layer
            let next_index = material_index + num_nodes as usize;
            let node_material_slice = &node_material_vector[material_index..next_index];
    
            // Update the material index for the next layer
            material_index = next_index;
    
            // Now we can call spawn_layer with x_positions and the material slice
            let transform = Transform::from_xyz(0.0, y_position, 0.0);
    
            Graph::spawn_layer(
                parent,
                &circle_mesh_handle,
                &annulus_mesh_handle,
                node_material_slice,
                &outline_material,
                &transform,
                &x_positions,
            );
        }
    }
}

impl Component for Graph {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
                // Step 1: Extract the Graph component (immutable borrow)
                let graph: Option<Graph> = {
                    let entity_ref: EntityRef<'_> = world.entity(entity);
                    entity_ref.get::<Graph>().cloned()
                };

                if let Some(graph) = graph {

                    let node_outer_radius = graph.node_outer_radius;
                    let node_inner_radius = graph.node_outer_radius - graph.node_border_thickness;
                    let circle_radius = node_inner_radius - 2.0;

                    // Step 2: Extract meshes and materials handles in limited scope
                    let (circle_mesh_handle, annulus_mesh_handle, node_material_vector, outline_material) = {
                        // Mutable borrow of `world` starts
                        let mut meshes = world.resource_mut::<Assets<Mesh>>();
                        let circle_mesh_handle = meshes.add(Mesh::from(Circle::new(circle_radius)));
                        let annulus_mesh_handle: Handle<Mesh> = meshes.add(Mesh::from(Annulus::new(
                            node_inner_radius,
                            node_outer_radius)
                        ));

                        let num_nodes = graph.num_nodes_per_layer.iter().sum();
                        let mut materials = world.resource_mut::<Assets<PulsingMaterial>>();

                        let node_material_vector: Vec<Handle<PulsingMaterial>> = if num_nodes > 0 {
                            (0..num_nodes)
                                .map(|i| {
                                    let phase = (i as f32) / ((num_nodes - 1).max(1) as f32) * 2.0 * std::f32::consts::TAU;
                                    materials.add(PulsingMaterial {
                                        color: HIGHLIGHT_COLOR.into(),
                                        phase,
                                    })
                                })
                                .collect()
                        } else {
                            Vec::new()
                        };

                        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
                        let outline_material  = materials.add(ColorMaterial::from(PRIMARY_COLOR));

                        (circle_mesh_handle, annulus_mesh_handle, node_material_vector, outline_material)
                    };

                    {
                        let mut commands = world.commands();

                        commands.entity(entity).with_children(|parent: &mut ChildBuilder<'_>| {
                            Graph::spawn(
                                parent,
                                circle_mesh_handle.clone(),
                                annulus_mesh_handle.clone(),
                                node_material_vector.clone(),
                                outline_material.clone(),
                                graph
                            );
                        });
                    }
                }
            },
        );
    }
}    

#[derive(Bundle)]
pub struct GraphBundle{
    graph : Graph,
    visibility : Visibility,
    transform : Transform
}

impl GraphBundle {
    pub fn new(
        inter_layer_distance : f32,
        num_nodes_per_layer : Vec<i32>,
        inter_node_distance : f32,
        node_outer_radius : f32,
        node_border_thickness : f32,
        translation : Vec3,
        scale : f32
    ) ->  Self {

        GraphBundle {
            graph : Graph::new(
                inter_layer_distance*scale,
                num_nodes_per_layer,
                inter_node_distance*scale,
                node_outer_radius*scale,
                node_border_thickness*scale
            ),
            visibility : Visibility::default(),
            transform : Transform::from_translation(translation)
        }
    }
}
