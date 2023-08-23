use bevy::prelude::*;

pub struct TargetGameplay;
impl Plugin for TargetGameplay {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (drift, age, die))
            .register_type::<TagA>()
            .register_type::<TagB>()
            .register_type::<Lifetime>()
            .register_type::<Velocity>();
    }
}
//Tag components make it easy for us to keep track of a few flavors of entity
#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct TagA {}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct TagB {}

#[derive(Component, Reflect, Default, Clone, Copy, Deref, DerefMut)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);

pub fn drift(mut q: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for (v, mut t) in &mut q {
        t.translation += **v * dt;
    }
}

#[derive(Component, Reflect, Default, Clone, Copy, Deref, DerefMut)]
#[reflect(Component)]
pub struct Lifetime(pub f32);

pub fn age(mut q: Query<&mut Lifetime>, time: Res<Time>) {
    let dt = time.delta_seconds();
    for mut l in &mut q {
        l.0 -= dt;
    }
}
pub fn die(q: Query<(Entity, &Lifetime)>, mut c: Commands) {
    for (e, l) in &q {
        if **l <= 0.0 {
            c.entity(e).despawn();
        }
    }
}

/*
// The original 'target gameplay'
//  Pretty simple, it spawns a number of tagged entities with
//  somewhat randomized properties, and maintains the count as they time out.
//  depended on the create(...) helper that has since been removed from the code.
pub fn do_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    a: Query<&TagA>,
    b: Query<&TagB>,
) {
    let mut rng: ThreadRng = Default::default();
    for _ in a.iter().count()..10 {
        let l: f32 = rng.sample::<f32, _>(Standard) * 6.0;
        let e = create(
            &mut commands,
            &mut meshes,
            &mut materials,
            l,
            Vec3::from((0., 0., 0.)),
            Color::rgb(1.0, 0.0, 1.0),
        );
        commands.entity(e).insert(TagA::default());
    }
    for _ in b.iter().count()..5 {
        let l: f32 = rng.sample::<f32, _>(Standard) * 1.0;
        let e = create(
            &mut commands,
            &mut meshes,
            &mut materials,
            l,
            Vec3::from((0., 0., 0.)),
            Color::rgb(0.0, 1.0, 1.0),
        );
        commands.entity(e).insert(TagB::default());
    }
}
*/
