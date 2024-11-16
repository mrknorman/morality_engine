use bevy::{
    prelude::*, 
	ecs::component::StorageType,
    sprite::{
        MaterialMesh2dBundle,
        Mesh2dHandle
    }
};

use crate::shaders::PulsingMaterial;

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
            
            /* 
            .add_systems(
                Update,
                (
                    Graph::advance_graph,
                    Graph::skip_controls,
                    Graph::play
                ).run_if(in_state(GraphSystemsActive::True))
            );
            */
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

impl GraphNode {

    pub fn pulse(
        mut node_query: Query<&GraphNode>
    ) {

        for node in node_query.iter_mut() {

        }
    }
}

#[derive(Clone)]
pub struct Graph {
    num_layers : i32,
    inter_layer_distance : f32,
    num_nodes_per_layer : Vec<i32>,
    inter_node_distance : f32,
    node_outer_radius : f32,
    node_border_thickness : f32,
}

impl Graph {
    pub fn new(
        num_layers : i32,
        inter_layer_distance : f32,
        num_nodes_per_layer : Vec<i32>,
        inter_node_distance : f32,
        node_outer_radius : f32,
        node_border_thickness : f32
    ) -> Self {
        Graph {
            num_layers,
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
        circle_mesh_handle: Handle<Mesh>,
        annulus_mesh_handle: Handle<Mesh>,
        material_handle: Handle<PulsingMaterial>,
        transform: Transform,
        x_positions: Vec<f32>,
    ) {

        for &x_position in &x_positions {
            // Set the transform for each node with the calculated x position
            let node_transform = Transform::from_translation(Vec3::new(
                x_position + transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ));

            // Spawn the circle bundle at the calculated position
            parent.spawn(
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(annulus_mesh_handle.clone()),
                    material: material_handle.clone(),
                    transform: node_transform.clone(),
                    ..default()
                },
            );
            parent.spawn(
                (
                    GraphNode,
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(circle_mesh_handle.clone()),
                        material: material_handle.clone(),
                        transform: node_transform,
                        ..default()
                    }
                ),
            );
        }
    }

    fn spawn(
        parent: &mut ChildBuilder<'_>,
        circle_mesh_handle: Handle<Mesh>,
        annulus_mesh_handle: Handle<Mesh>,
        material_handle: Handle<PulsingMaterial>,
        graph : Graph
    ) {
        let base_spacing = graph.node_outer_radius * 2.0 + graph.inter_node_distance;

        let center_offset = ((graph.num_layers - 1) as f32) / 2.0 * graph.inter_layer_distance;

        for (layer_index, &num_nodes) in graph.num_nodes_per_layer.iter().enumerate() {
            let y_position = layer_index as f32 * graph.inter_layer_distance - center_offset;

            // Compute x positions
            let x_positions = Graph::compute_x_positions(num_nodes, base_spacing);

            // Now we can call spawn_graph_layer with x_positions
            let transform = Transform::from_xyz(0.0, y_position, 0.0);

            Graph::spawn_layer(
                parent,
                circle_mesh_handle.clone(),
                annulus_mesh_handle.clone(),
                material_handle.clone(),
                transform,
                x_positions,
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
                    let (circle_mesh_handle, annulus_mesh_handle, material_handle) = {
                        // Mutable borrow of `world` starts
                        let mut meshes = world.resource_mut::<Assets<Mesh>>();
                        let circle_mesh_handle = meshes.add(Mesh::from(Circle::new(circle_radius)));
                        let annulus_mesh_handle: Handle<Mesh> = meshes.add(Mesh::from(Annulus::new(
                            node_inner_radius,
                            node_outer_radius)
                        ));

                        let asset_server = world.resource::<AssetServer>();
                        let texture: Handle<Image> = asset_server.load("branding/icon.png");

                        let mut materials = world.resource_mut::<Assets<PulsingMaterial>>();
                        let material_handle = materials.add(PulsingMaterial {
                            color: LinearRgba::new(1.0, 1.0, 1.0, 1.0),
                            color_texture: Some(texture),
                        });

                        // Mutable borrow of `world` ends
                        (circle_mesh_handle, annulus_mesh_handle, material_handle)
                    };

                    // Step 3: Use commands in a separate scope
                    {
                        // Mutable borrow of `world` starts
                        let mut commands = world.commands();

                        commands.entity(entity).with_children(|parent: &mut ChildBuilder<'_>| {
                            // Pass the handles to your spawn function
                            Graph::spawn(
                                parent,
                                circle_mesh_handle.clone(),
                                annulus_mesh_handle.clone(),
                                material_handle.clone(),
                                graph
                            );
                        });
                        // Mutable borrow of `world` ends
                    }
                    // All mutable borrows of `world` are now non-overlapping
                }
            },
        );
    }
}    

#[derive(Bundle)]
pub struct GraphBundle{
    graph : Graph,
    visibility : VisibilityBundle,
    transform : TransformBundle
}

impl GraphBundle {
    pub fn new(
        num_layers : i32,
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
                num_layers,
                inter_layer_distance*scale,
                num_nodes_per_layer,
                inter_node_distance*scale,
                node_outer_radius*scale,
                node_border_thickness*scale
            ),
            visibility : VisibilityBundle::default(),
            transform : TransformBundle::from_transform(
                Transform::from_translation(translation)
            )
        }
    }
}
