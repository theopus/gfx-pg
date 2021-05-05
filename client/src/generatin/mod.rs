use std::fmt;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rand;
use rand::{Rng, SeedableRng};
use rand::distributions::Distribution;
use rand_distr;
use rand_distr::Normal;

use rx::utils::file_system;

struct Person {
    first_name: String,
    last_name: String,
    height: u32,
    sex: u8,
}

impl Person {
    #[allow(dead_code)]
    pub fn new(first_name: String, last_name: String, height: u32, sex: u8) -> Self {
        Person {
            first_name,
            last_name,
            height,
            sex,
        }
    }
}

impl fmt::Debug for Person {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Person")
            .field("first_name", &self.first_name)
            .field("last_name", &self.last_name)
            .field("height", &self.height)
            .field(
                "sex",
                match &self.sex {
                    0 => &"male",
                    1 => &"female",
                    _ => &"other",
                },
            )
            .finish()
    }
}

trait FromRng {
    fn from_rng(rng: &mut impl Rng) -> Self;
}

impl FromRng for Person {
    fn from_rng(rng: &mut impl Rng) -> Self {
        let male_height_distribution: Normal<f64> = rand_distr::Normal::new(178.4, 7.58).unwrap();
        let female_height_distribution: Normal<f64> = rand_distr::Normal::new(164.7, 7.07).unwrap();

        let sex = rng.gen_range(0..2);
        let height: u32 = match sex {
            0 => male_height_distribution.sample(rng),
            1 => female_height_distribution.sample(rng),
            _ => male_height_distribution.sample(rng),
        } as u32;

        let name_offset = rng.gen_range(0..1000);
        let last_name_offset = rng.gen_range(0..1000);

        let first_name = file_system::read_lines_r(&["assets", "dict", "first_names.csv"])
            .skip(name_offset)
            .next()
            .unwrap()
            .unwrap();
        let last_name = file_system::read_lines_r(&["assets", "dict", "last_names.csv"])
            .skip(last_name_offset)
            .next()
            .unwrap()
            .unwrap();

        Person {
            first_name,
            last_name,
            height,
            sex,
        }
    }
}

mod spc {
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};

    use rx::specs;
    use rx::specs::{Component, prelude::*};
    #[allow(unused_imports)]
    use rx::specs::Join;

    struct DecisionSystem;

    impl<'a> specs::System<'a> for DecisionSystem {
        type SystemData = WriteStorage<'a, Status>;

        fn run(&mut self, data: Self::SystemData) {
            #[allow(unused_imports)]
            use rx::specs::Join;
            let status = data;
            for s in status.join() {
                info!("{:?}", &s);
            }
        }
    }

    #[derive(Component, Debug, Default)]
    #[storage(VecStorage)]
    struct Status {
        wealth: f32,
        mood: f32,
    }

    #[test]
    fn test_sim() {
        let mut world = specs::World::new();
        world.register::<Status>();
        let mut dispatcher = specs::DispatcherBuilder::new()
            .with(DecisionSystem, "decision", &[]).build();

        dispatcher.dispatch(&world)
    }
}

#[test]
fn test() {
    use itertools::Itertools;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;

    crate::init_log();


    let mut rng = Pcg64::from_seed(Seeder::from("ira").make_seed());
    // let mut rng = Pcg64::from_entropy();

    for p in (1..100)
        .map(|_n| Person::from_rng(&mut rng))
        .sorted_by(|a, b| Ord::cmp(&a.last_name, &b.last_name)) {
        info!("{:?}", p);
    }
}
