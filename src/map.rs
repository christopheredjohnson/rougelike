use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct Room {
    pub id: usize,
    pub bounds: Rect, // Original BSP split area
    pub inner: Rect,  // Carved room within bounds
}



impl Rect {
    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn subdivide(&self, rng: &mut impl Rng) -> Option<(Rect, Rect)> {
        let min_size = 6;

        let can_split_h = self.height > min_size * 2;
        let can_split_v = self.width > min_size * 2;

        if !can_split_h && !can_split_v {
            return None;
        }

        let split_horizontal = if can_split_h && can_split_v {
            rng.gen_bool(0.5)
        } else {
            can_split_h
        };

        if split_horizontal {
            let max_split = self.height - min_size;
            let min_split = min_size;
            if min_split < max_split {
                let split = rng.gen_range(min_split..max_split);
                Some((
                    Rect {
                        x: self.x,
                        y: self.y,
                        width: self.width,
                        height: split,
                    },
                    Rect {
                        x: self.x,
                        y: self.y + split,
                        width: self.width,
                        height: self.height - split,
                    },
                ))
            } else {
                None
            }
        } else {
            let max_split = self.width - min_size;
            let min_split = min_size;
            if min_split < max_split {
                let split = rng.gen_range(min_split..max_split);
                Some((
                    Rect {
                        x: self.x,
                        y: self.y,
                        width: split,
                        height: self.height,
                    },
                    Rect {
                        x: self.x + split,
                        y: self.y,
                        width: self.width - split,
                        height: self.height,
                    },
                ))
            } else {
                None
            }
        }
    }
}

pub fn bsp_split(rect: Rect, depth: u32, rng: &mut impl Rng) -> Vec<Room> {
    let mut leaves = vec![rect];
    for _ in 0..depth {
        let mut next = Vec::new();
        for r in &leaves {
            if let Some((a, b)) = r.subdivide(rng) {
                next.push(a);
                next.push(b);
            } else {
                next.push(*r);
            }
        }
        leaves = next;
    }

    leaves
        .into_iter()
        .enumerate()
        .map(|(i, bounds)| {
            let margin = 1;
            let inner = Rect {
                x: bounds.x + margin,
                y: bounds.y + margin,
                width: bounds.width - margin * 2,
                height: bounds.height - margin * 2,
            };
            Room {
                id: i,
                bounds,
                inner,
            }
        })
        .collect()
}