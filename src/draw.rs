use cgmath::{InnerSpace, Matrix3, Rad, Vector3};
use crossterm::terminal::size;
use nalgebra::{AbstractRotation, Point3, Rotation3, Vector3 as nVector3};
use parry3d::transformation::convex_hull;
use std::io::{stdout, Write};

/// The thing that makes scanline work
fn intersections(rowy: i32, edges: &Vec<((i32, i32), (i32, i32))>) -> Vec<i32> {
    let mut intersections = vec![];

    for edge in edges {
        // Horizontal lines == bad
        if edge.0 .1 == edge.1 .1 {
            continue;
        }

        // Sort the edges for consistency
        let (p1, p2) = if edge.0 .1 <= edge.1 .1 {
            (edge.0, edge.1)
        } else {
            (edge.1, edge.0)
        };

        // Check if an intersection occurs
        if rowy >= p1.1 && rowy < p2.1 {
            // uhhh
            let x_intersec = p1.0 + (rowy - p1.1) * (p2.0 - p1.0) / (p2.1 - p1.1);
            intersections.push(x_intersec);
        }
    }

    intersections
}

/// The screen
pub struct Screen {
    pub pxs: Vec<Vec<(u8, u8, u8)>>,
    pub lightdir: Vector3<f32>,
    pub camdir: (f32, f32, f32),
    pub campos: (f32, f32, f32),
}

impl Screen {
    /// Builder of screens
    pub fn new(lightdir: (f32, f32, f32)) -> Self {
        let tsize = size().unwrap();
        Screen {
            pxs: vec![vec![(0, 0, 0); tsize.0 as usize]; tsize.1 as usize * 2],
            lightdir: Vector3::new(lightdir.0, lightdir.1, lightdir.2).normalize(),
            camdir: (0.0, 0.0, 0.0),
            campos: (0.0, 0.0, 0.0),
        }
    }
    /// Draw the pixels to the screen
    pub fn write(&mut self) {
        let debug = false;
        // CRT monitor ass rendering
        let dbgrows = [
            format!(
                "Camera Pos: ({:.1}, {:.1}, {:.1})",
                self.campos.0, self.campos.1, self.campos.2
            ),
            format!(
                "Camera Rot: ({:.1}, {:.1}, {:.1})",
                self.camdir.0, self.camdir.1, self.camdir.2
            ),
        ];
        for i in 0..self.pxs.len() / 2 {
            for col in 0..self.pxs[0].len() {
                if i < 2 && col < dbgrows[i].len() && debug {
                    print!(
                        "\x1b[38;2;255;255;255;48;2;0;0;0m{}",
                        dbgrows[i].chars().collect::<Vec<char>>()[col]
                    );
                } else {
                    print!(
                        "\x1b[38;2;{};{};{};48;2;{};{};{}m▀",
                        self.pxs[i * 2][col].0,
                        self.pxs[i * 2][col].1,
                        self.pxs[i * 2][col].2,
                        self.pxs[i * 2 + 1][col].0.clamp(0, 255),
                        self.pxs[i * 2 + 1][col].1.clamp(0, 255),
                        self.pxs[i * 2 + 1][col].2.clamp(0, 255),
                    );
                }
            }
        }
        stdout().flush().unwrap();
    }
    /// Clear the screen
    pub fn clear(&mut self) {
        let tsize = (self.pxs[0].len(), self.pxs.len());
        self.pxs = vec![vec![(0, 0, 0); tsize.0]; tsize.1];
    }
    /// Scan-line fill algorithm
    pub fn polygon(&mut self, points: Vec<(i32, i32)>, colour: (u8, u8, u8)) {
        // Allocate room for the pixels/
        let mut pxs = vec![];

        // Get the boundry's of the scan-line operation (for optimisation)
        let ymin = points.iter().map(|p| p.1).min().unwrap();
        let ymax = points.iter().map(|p| p.1).max().unwrap();

        // Get the edges
        let mut edges = Vec::new();
        for i in 0..points.len() {
            edges.push((points[i], points[(i + 1) % points.len()]));
        }

        for line in ymin..=ymax {
            let mut intersecs = intersections(line, &edges);

            intersecs.sort(); // Do the cool smart thing that makes this not O(n^2)

            for pair in intersecs.chunks(2) {
                if pair.len() == 2 {
                    let (start, end) = (pair[0], pair[1]);
                    // Fill in where it's in the polygon
                    for x in start..=end {
                        pxs.push((x, line));
                    }
                }
            }
        }

        // Render the pixels
        for px in pxs {
            if px.0 == px.0.clamp(0, self.pxs[0].len() as i32 - 1)
                && px.1 == px.1.clamp(0, self.pxs.len() as i32 - 1)
            {
                self.pxs[px.1 as usize][px.0 as usize] = colour;
                // Formerly the vertex highlighter (R.I.P)
                // if points.contains(&px) {
                //     self.pxs[px.1 as usize][px.0 as usize] = (255, 0, 255);
                // }
            }
        }
    }

    /// Creates a mesh from points
    pub fn polyhedron(
        &mut self,
        vertices: Vec<Point3<f32>>,
        pos: (f32, f32, f32),
        rotation: (f32, f32, f32),
        scale: (f32, f32, f32),
        colour: (u8, u8, u8),
        noshade: bool,
    ) {
        // Scaling thing
        let posv = Vector3::new(
            pos.0 - self.campos.0,
            pos.1 - self.campos.1,
            pos.2 - self.campos.2 / 100.0,
        );
        let fz = if posv[2] > -0.01 && posv[2] <= 0.0 {
            posv[2] + 1.01
        } else {
            posv[2] + 1.0
        };
        let size = [scale.0, scale.1, scale.2]
            .iter()
            .map(|s| *s / fz)
            .collect::<Vec<f32>>();

        let faces = convex_hull(&vertices).1; // Took hours to find parry3d

        // Switch library's for some reason
        let mut cvertices = vertices
            .iter()
            .map(|vert| Vector3::new(vert.x, vert.y, vert.z))
            .collect::<Vec<Vector3<f32>>>();

        // Apply scaling
        cvertices = cvertices
            .iter()
            .map(|vert| Vector3::new(vert[0] * size[0], vert[1] * size[1], vert[2] * size[2]))
            .collect::<Vec<Vector3<f32>>>();

        // Convert to radians
        let rot = (
            rotation.0.to_radians() - self.camdir.0.to_radians(),
            rotation.1.to_radians() - self.camdir.1.to_radians(),
            rotation.2.to_radians() - self.camdir.2.to_radians(),
        );

        // Make the rotation thing
        let rotx = Matrix3::from_angle_x(Rad(rot.0));
        let roty = Matrix3::from_angle_y(Rad(rot.1));
        let rotz = Matrix3::from_angle_z(Rad(rot.2));

        // Apply rotation and position
        let verts: Vec<Vector3<f32>> = cvertices
            .iter()
            .map(|point| rotx * roty * rotz * point + posv)
            .collect();

        // Occlussion
        let mut face_depths: Vec<(usize, f32)> = faces
            .iter()
            .enumerate()
            .map(|(idx, face)| {
                let face_verts = face
                    .iter()
                    .map(|&i| verts[i as usize])
                    .collect::<Vec<Vector3<f32>>>();
                // This was just trial and error pretty much

                let edge1 = face_verts[1] - face_verts[0];
                let edge2 = face_verts[2] - face_verts[0];
                let normal = edge1.cross(edge2).normalize();
                let avg_depth = normal.z;

                (idx, avg_depth)
            })
            .collect();

        // Sort the faces
        face_depths.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Reverse the faces
        let final_faces = face_depths[0..faces.len()].iter().rev();

        // Render the three closest faces
        for (idx, _) in final_faces {
            // Wierd rust memory garbage
            let face = &faces[*idx];
            // Iterators woooooooooo
            let face_verts = face
                .iter()
                .map(|&i| verts[i as usize])
                .collect::<Vec<Vector3<f32>>>();

            // Woosh woosh lighting
            let edge1 = face_verts[1] - face_verts[0];
            let edge2 = face_verts[2] - face_verts[0];
            let normal = edge1.cross(edge2).normalize();

            // The cool linear algebra
            let mut brightness =
                (normal.dot(self.lightdir).max(0.0) * 220.0 + 34.0).min(255.0) as u8;

            // If noshade is not, then no shade
            if noshade {
                brightness = 255
            }

            // Deletus of z axis
            let face_verts_2d = face_verts
                .iter()
                .map(|v| (v[0] as i32, v[1] as i32))
                .collect::<Vec<(i32, i32)>>();

            // Draw the polygon
            self.polygon(
                face_verts_2d,
                (
                    (colour.0 as f32 / 255.0 * brightness as f32) as u8,
                    (colour.1 as f32 / 255.0 * brightness as f32) as u8,
                    (colour.2 as f32 / 255.0 * brightness as f32) as u8,
                ),
            );
        }
    }
    /// Box
    pub fn cube(
        &mut self,
        pos: (f32, f32, f32),
        rotation: (f32, f32, f32),
        scale: (f32, f32, f32),
        colour: (u8, u8, u8),
        noshade: bool,
    ) {
        let vertices = vec![
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(1.0, -1.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, 1.0, -1.0),
            Point3::new(1.0, -1.0, -1.0),
            Point3::new(-1.0, 1.0, -1.0),
            Point3::new(-1.0, -1.0, -1.0),
        ];
        self.polyhedron(vertices, pos, rotation, scale, colour, noshade);
    }
    /// Egypt copycat
    pub fn pyramid(
        &mut self,
        pos: (f32, f32, f32),
        rotation: (f32, f32, f32),
        scale: (f32, f32, f32),
        colour: (u8, u8, u8),
        noshade: bool,
    ) {
        let vertices = vec![
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(-1.0, 1.0, 1.0),
            Point3::new(0.0, -1.0, 0.0),
            Point3::new(1.0, 1.0, -1.0),
            Point3::new(-1.0, 1.0, -1.0),
        ];
        self.polyhedron(vertices, pos, rotation, scale, colour, noshade);
    }
    /// Icosphere's
    pub fn icosphere(
        &mut self,
        pos: (f32, f32, f32),
        rotation: (f32, f32, f32),
        radius: f32,
        subsdivisions: u32,
        colour: (u8, u8, u8),
        noshade: bool,
    ) {
        let xrect = vec![
            Point3::new(-1.618, 0.0, 1.0),
            Point3::new(-1.618, 0.0, -1.0),
            Point3::new(1.618, 0.0, 1.0),
            Point3::new(1.618, 0.0, -1.0),
        ];
        let yrect = vec![
            Point3::new(1.0, 1.618, 0.0),
            Point3::new(-1.0, 1.618, 0.0),
            Point3::new(1.0, -1.618, 0.0),
            Point3::new(-1.0, -1.618, 0.0),
        ];
        let zrect = vec![
            Point3::new(0.0, 1.0, 1.618),
            Point3::new(0.0, -1.0, 1.618),
            Point3::new(0.0, 1.0, -1.618),
            Point3::new(0.0, -1.0, -1.618),
        ];
        let mut vertices = vec![];
        vertices.extend(xrect);
        vertices.extend(yrect);
        vertices.extend(zrect);
        for _ in 0..subsdivisions {
            let faces = convex_hull(&vertices).1;

            for face in &faces {
                let v1 = nVector3::new(
                    vertices[face[0] as usize][0],
                    vertices[face[0] as usize][1],
                    vertices[face[0] as usize][2],
                );
                let v2 = nVector3::new(
                    vertices[face[1] as usize][0],
                    vertices[face[1] as usize][1],
                    vertices[face[1] as usize][2],
                );
                let v3 = nVector3::new(
                    vertices[face[2] as usize][0],
                    vertices[face[2] as usize][1],
                    vertices[face[2] as usize][2],
                );

                let v4 = v1.slerp(&v2, 0.5);
                let v5 = v2.slerp(&v3, 0.5);
                let v6 = v3.slerp(&v1, 0.5);

                vertices.push(Point3::from(v4));
                vertices.push(Point3::from(v5));
                vertices.push(Point3::from(v6));
            }
        }
        self.polyhedron(
            vertices,
            pos,
            rotation,
            (radius, radius, radius),
            colour,
            noshade,
        );
    }
}
