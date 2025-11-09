use bevy::prelude::*;
use crate::components::player::BoundaryWall;
use crate::systems::config::{BOUNDARY_LEFT, BOUNDARY_RIGHT, BOUNDARY_TOP, BOUNDARY_BOTTOM, BOUNDARY_WALL_THICKNESS};

/// Spawns the visual boundary walls (red walls on left/right, green line on top)
pub fn spawn_boundaries(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Left wall (red)
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(BOUNDARY_WALL_THICKNESS, BOUNDARY_TOP - BOUNDARY_BOTTOM))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))), // Red
        Transform::from_xyz(BOUNDARY_LEFT, (BOUNDARY_TOP + BOUNDARY_BOTTOM) / 2.0, 0.0),
        BoundaryWall,
    ));

    // Right wall (red)
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(BOUNDARY_WALL_THICKNESS, BOUNDARY_TOP - BOUNDARY_BOTTOM))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))), // Red
        Transform::from_xyz(BOUNDARY_RIGHT, (BOUNDARY_TOP + BOUNDARY_BOTTOM) / 2.0, 0.0),
        BoundaryWall,
    ));

    // Top boundary line (green)
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(BOUNDARY_RIGHT - BOUNDARY_LEFT, BOUNDARY_WALL_THICKNESS))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0))), // Green
        Transform::from_xyz((BOUNDARY_LEFT + BOUNDARY_RIGHT) / 2.0, BOUNDARY_TOP, 0.0),
        BoundaryWall,
    ));
}

