use bevy::prelude::*;

use crate::selection::Selectable;

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize_or_zero(),
        }
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    pub fn is_valid(&self) -> bool {
        self.direction.length_squared() > f32::EPSILON
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RayHit {
    pub entity: Entity,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub priority: f32,
    pub selectable: bool,
}

impl RayHit {
    pub fn new(
        entity: Entity,
        distance: f32,
        point: Vec3,
        normal: Vec3,
        priority: f32,
        selectable: bool,
    ) -> Self {
        Self {
            entity,
            distance,
            point,
            normal,
            priority,
            selectable,
        }
    }
}

/// Utility functions and algorithms for ray casting operations.
///
/// This struct provides static methods for creating rays from screen coordinates
/// and testing intersections with various geometric primitives.
pub struct RayCaster;

impl RayCaster {
    /// Default field of view for ray casting when camera projection is not accessible.
    const DEFAULT_FOV_Y: f32 = std::f32::consts::FRAC_PI_4; // 45 degrees

    /// Creates a ray from screen coordinates through the camera viewport.
    ///
    /// This method converts 2D screen coordinates to a 3D ray in world space,
    /// taking into account the camera's position, orientation, and field of view.
    ///
    /// # Arguments
    /// * `screen_pos` - Screen coordinates (in pixels)
    /// * `camera_transform` - Camera's world transform
    /// * `window` - Window for viewport dimensions
    ///
    /// # Returns
    /// A ray in world space, or None if the operation fails
    pub fn screen_to_world_ray(
        screen_pos: Vec2,
        camera_transform: &GlobalTransform,
        window: &Window,
    ) -> Option<Ray> {
        let window_size = Vec2::new(window.width(), window.height());

        // Validate window dimensions
        if window_size.x <= 0.0 || window_size.y <= 0.0 {
            warn!("Invalid window dimensions: {:?}", window_size);
            return None;
        }

        // Convert screen coordinates to normalized device coordinates (-1 to 1)
        let ndc = Vec2::new(
            (screen_pos.x / window_size.x) * 2.0 - 1.0,
            1.0 - (screen_pos.y / window_size.y) * 2.0, // Flip Y (screen Y increases downward)
        );

        debug!(
            "Screen to NDC conversion: screen={:?}, window_size={:?}, ndc={:?}",
            screen_pos, window_size, ndc
        );

        // Extract camera orientation vectors
        let camera_pos = camera_transform.translation();
        let camera_forward = *camera_transform.forward();
        let camera_right = *camera_transform.right();
        let camera_up = *camera_transform.up();

        debug!(
            "Camera vectors: pos={:?}, forward={:?}, right={:?}, up={:?}",
            camera_pos, camera_forward, camera_right, camera_up
        );

        // Calculate field of view and aspect ratio
        let aspect_ratio = window.width() / window.height();
        let fov_x = Self::DEFAULT_FOV_Y * aspect_ratio;

        // Convert NDC to camera space direction
        let x_offset = ndc.x * (fov_x * 0.5).tan();
        let y_offset = ndc.y * (Self::DEFAULT_FOV_Y * 0.5).tan();

        debug!(
            "Ray calculation: aspect_ratio={:.3}, fov_x={:.3}, x_offset={:.3}, y_offset={:.3}",
            aspect_ratio, fov_x, x_offset, y_offset
        );

        let ray_direction = camera_forward + camera_right * x_offset + camera_up * y_offset;

        let ray = Ray::new(camera_pos, ray_direction);

        // Validate the generated ray
        if ray.is_valid() {
            debug!(
                "Created valid ray: origin={:?}, direction={:?}",
                ray.origin, ray.direction
            );
            Some(ray)
        } else {
            warn!(
                "Generated invalid ray: origin={:?}, direction={:?}",
                ray.origin, ray.direction
            );
            None
        }
    }

    /// Tests ray intersection with an axis-aligned bounding box (AABB).
    ///
    /// This method uses the slab method for efficient AABB intersection testing.
    /// It handles edge cases like zero direction components and returns the distance
    /// to the nearest intersection point.
    ///
    /// # Arguments
    /// * `ray` - The ray to test intersection with
    /// * `aabb_min` - Minimum corner of the AABB
    /// * `aabb_max` - Maximum corner of the AABB
    ///
    /// # Returns
    /// Distance to intersection point, or None if no intersection occurs
    pub fn ray_aabb_intersection(ray: &Ray, aabb_min: Vec3, aabb_max: Vec3) -> Option<f32> {
        // Handle potential division by zero by using a small epsilon
        let inv_dir = Vec3::new(
            if ray.direction.x.abs() < f32::EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray.direction.x
            },
            if ray.direction.y.abs() < f32::EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray.direction.y
            },
            if ray.direction.z.abs() < f32::EPSILON {
                f32::INFINITY
            } else {
                1.0 / ray.direction.z
            },
        );

        let t1 = (aabb_min - ray.origin) * inv_dir;
        let t2 = (aabb_max - ray.origin) * inv_dir;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_near = t_min.max_element();
        let t_far = t_max.min_element();

        // Check if intersection occurs and is in front of the ray
        if t_near <= t_far && t_far >= 0.0 {
            if t_near >= 0.0 {
                Some(t_near)
            } else {
                Some(t_far)
            }
        } else {
            None
        }
    }

    /// Computes an approximate AABB for a mesh entity based on its transform.
    ///
    /// This method creates a bounding box around an entity's position using
    /// a uniform scale factor. For more accurate results, consider using
    /// the actual mesh bounds when available.
    ///
    /// # Arguments
    /// * `transform` - World transform of the entity
    /// * `scale` - Uniform scale factor for the bounding box
    ///
    /// # Returns
    /// Tuple of (min_corner, max_corner) representing the AABB
    pub fn get_entity_aabb(transform: &GlobalTransform, scale: f32) -> (Vec3, Vec3) {
        let position = transform.translation();
        let transform_scale = transform.compute_transform().scale;
        let actual_scale = scale * transform_scale.max_element();
        let extent = Vec3::splat((actual_scale * 0.5).max(0.01)); // Ensure minimum size
        (position - extent, position + extent)
    }

    /// Gets accurate bounding box dimensions for different entity types.
    ///
    /// This method determines the appropriate bounding box size based on
    /// the entity's Selectable ID, accounting for the actual mesh dimensions
    /// used in the game.
    ///
    /// # Arguments
    /// * `selectable` - The Selectable component containing entity information
    ///
    /// # Returns
    /// The appropriate scale factor for this entity type
    pub fn get_bbox_scale_for_entity(selectable: &Selectable) -> f32 {
        if let Some(id) = &selectable.id {
            if id.starts_with("color_") {
                // Color panel squares: 0.5 units + margin
                0.6
            } else if id.contains("_face")
                || id.contains("front_")
                || id.contains("back_")
                || id.contains("left_")
                || id.contains("right_")
                || id.contains("top_")
                || id.contains("bottom_")
            {
                // Cube faces: 0.9 scale * (2.0/3.0) base size + margin
                0.7
            } else {
                // Default fallback
                0.5
            }
        } else {
            // No ID - use conservative small size
            0.3
        }
    }

    /// Performs ray casting against all selectable entities and returns sorted hits.
    ///
    /// This method tests the ray against all entities with Selectable components,
    /// performs intersection tests, and returns hits sorted by priority and distance.
    ///
    /// # Arguments
    /// * `ray` - The ray to cast
    /// * `selectable_query` - Query for selectable entities
    ///
    /// # Returns
    /// Vector of ray hits, sorted by priority (descending) then distance (ascending)
    pub fn cast_ray(
        ray: &Ray,
        selectable_query: &Query<(Entity, &GlobalTransform, &Selectable)>,
    ) -> Vec<RayHit> {
        let mut hits = Vec::new();
        debug!(
            "Ray casting from {:?} in direction {:?}",
            ray.origin, ray.direction
        );

        for (entity, transform, selectable) in selectable_query.iter() {
            // Skip disabled selectables
            if !selectable.enabled {
                continue;
            }

            // Get appropriate bounding box size for this entity type
            let bbox_scale = Self::get_bbox_scale_for_entity(selectable);
            let (aabb_min, aabb_max) = Self::get_entity_aabb(transform, bbox_scale);

            debug!(
                "Testing entity {:?} (id: {:?}, priority: {:.1}) at position {:?}",
                entity,
                selectable.id,
                selectable.priority,
                transform.translation()
            );
            debug!(
                "  AABB (scale={:.2}): min={:?}, max={:?}",
                bbox_scale, aabb_min, aabb_max
            );

            if let Some(distance) = Self::ray_aabb_intersection(ray, aabb_min, aabb_max) {
                let hit_point = ray.at(distance);
                let normal = (hit_point - transform.translation()).normalize_or_zero();

                debug!(
                    "  HIT: distance={:.3}, point={:?}, priority={:.1}",
                    distance, hit_point, selectable.priority
                );

                hits.push(RayHit::new(
                    entity,
                    distance,
                    hit_point,
                    normal,
                    selectable.priority,
                    true,
                ));
            } else {
                debug!("  MISS");
            }
        }

        // Sort hits by priority (descending) then by distance (ascending)
        hits.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    a.distance
                        .partial_cmp(&b.distance)
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
        });

        debug!("Ray casting found {} hits", hits.len());
        for (i, hit) in hits.iter().enumerate() {
            debug!(
                "  Hit {}: entity={:?}, distance={:.3}, priority={:.1}",
                i, hit.entity, hit.distance, hit.priority
            );
        }

        // Filter out hits that might be too close together (potential precision issues)
        let original_count = hits.len();
        let filtered_hits = Self::filter_precision_hits(hits);

        if filtered_hits.len() != original_count {
            debug!(
                "Filtered {} potentially overlapping hits",
                original_count - filtered_hits.len()
            );
        }

        filtered_hits
    }

    /// Filters out hits that might be causing precision issues.
    ///
    /// This method removes hits that are very close to higher-priority hits,
    /// which can happen when rays intersect overlapping or closely positioned entities.
    ///
    /// # Arguments
    /// * `hits` - Vector of ray hits sorted by priority and distance
    ///
    /// # Returns
    /// Filtered vector with precision issues removed
    pub fn filter_precision_hits(hits: Vec<RayHit>) -> Vec<RayHit> {
        if hits.is_empty() {
            return hits;
        }

        let mut filtered_hits = Vec::new();
        let distance_threshold = 0.1; // Minimum distance between different priority hits

        for hit in hits {
            let should_include = filtered_hits.iter().all(|existing_hit: &RayHit| {
                // Always include if same priority level
                if (hit.priority - existing_hit.priority).abs() < f32::EPSILON {
                    true
                } else {
                    // Only include if sufficiently far from higher priority hits
                    if hit.priority < existing_hit.priority {
                        (hit.distance - existing_hit.distance).abs() > distance_threshold
                    } else {
                        // This hit has higher priority, so always include
                        true
                    }
                }
            });

            if should_include {
                filtered_hits.push(hit);
            } else {
                debug!(
                    "Filtered out hit at distance {:.3} due to proximity to higher priority hit",
                    hit.distance
                );
            }
        }

        filtered_hits
    }
}
