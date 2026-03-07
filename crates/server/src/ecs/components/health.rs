use shipyard::Component;

#[allow(dead_code)]
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub min: f32,
    pub max: f32,
}
