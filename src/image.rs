use glm::Vec3;
use std::fs::File;
use std::io::Write;

pub struct Image {
    pub width: usize,
    pub height: usize,
    data: Vec<Vec3>,
}

impl Image {
    pub fn new(width: usize, height: usize, color: Vec3) -> Self {
        Self {
            width,
            height,
            data: vec![color; width * height],
        }
    }

    pub fn set(&mut self, u: usize, v: usize, color: Vec3) {
        let v = self.height - 1 - v;
        self.data[self.width * v + u] = color;
    }

    pub fn write(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        file.write("P6\n".as_bytes()).unwrap();
        file.write(format!("{} {}\n", self.width, self.height).as_bytes())
            .unwrap();
        file.write("255\n".as_bytes()).unwrap();

        let data = self
            .data
            .iter()
            .flat_map(|color| {
                [color.x, color.y, color.z]
                    .into_iter()
                    .map(|x| (255.0 * x).round() as u8)
            })
            .collect::<Vec<_>>();

        file.write(&data).unwrap();
    }
}
