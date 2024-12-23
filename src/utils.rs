/// The thing that makes scanline work
pub fn intersections(rowy: i32, edges: &Vec<((i32, i32), (i32, i32))>) -> Vec<i32> {
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
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    if s <= 0.0 {
        return (v, v, v);
    }
    let mut i = (h * 6.0).trunc();
    let f = (h * 6.0) - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    i %= 6.0;
    match i as i32 {
        0 => return (v, t, p),
        1 => return (q, v, p),
        2 => return (p, v, t),
        3 => return (p, q, v),
        4 => return (t, p, v),
        5 => return (v, p, q),
        _ => return (0.0, 0.0, 0.0),
    }
}
