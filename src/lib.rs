use std::cell::RefCell;
use std::f32::consts::PI;
use std::slice;

const ON_TREE: u8 = 0;
const FALLING: u8 = 1;
const ON_GROUND: u8 = 2;

thread_local! {
    static SIM: RefCell<SakuraSim> = RefCell::new(SakuraSim::new());
}

struct SakuraSim {
    petal_count: usize,
    rng_state: u32,
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
    pos_z: Vec<f32>,
    vel_x: Vec<f32>,
    vel_y: Vec<f32>,
    vel_z: Vec<f32>,
    rot_x: Vec<f32>,
    rot_y: Vec<f32>,
    rot_z: Vec<f32>,
    rot_speed_x: Vec<f32>,
    rot_speed_y: Vec<f32>,
    rot_speed_z: Vec<f32>,
    tree_pos_x: Vec<f32>,
    tree_pos_y: Vec<f32>,
    tree_pos_z: Vec<f32>,
    tree_rot_x: Vec<f32>,
    tree_rot_y: Vec<f32>,
    tree_rot_z: Vec<f32>,
    phase: Vec<f32>,
    scale: Vec<f32>,
    brightness: Vec<f32>,
    ground_offset: Vec<f32>,
    fall_delay: Vec<i32>,
    state: Vec<u8>,
    active_indices: Vec<u32>,
    active_count: usize,
    activated_indices: Vec<u32>,
    activated_count: usize,
    landed_indices: Vec<u32>,
    landed_count: usize,
}

impl SakuraSim {
    fn new() -> SakuraSim {
        SakuraSim {
            petal_count: 0,
            rng_state: 1,
            pos_x: Vec::new(),
            pos_y: Vec::new(),
            pos_z: Vec::new(),
            vel_x: Vec::new(),
            vel_y: Vec::new(),
            vel_z: Vec::new(),
            rot_x: Vec::new(),
            rot_y: Vec::new(),
            rot_z: Vec::new(),
            rot_speed_x: Vec::new(),
            rot_speed_y: Vec::new(),
            rot_speed_z: Vec::new(),
            tree_pos_x: Vec::new(),
            tree_pos_y: Vec::new(),
            tree_pos_z: Vec::new(),
            tree_rot_x: Vec::new(),
            tree_rot_y: Vec::new(),
            tree_rot_z: Vec::new(),
            phase: Vec::new(),
            scale: Vec::new(),
            brightness: Vec::new(),
            ground_offset: Vec::new(),
            fall_delay: Vec::new(),
            state: Vec::new(),
            active_indices: Vec::new(),
            active_count: 0,
            activated_indices: Vec::new(),
            activated_count: 0,
            landed_indices: Vec::new(),
            landed_count: 0,
        }
    }

    fn reset(&mut self, count: u32) {
        let count = count as usize;
        self.petal_count = count;
        self.active_count = 0;
        self.activated_count = 0;
        self.landed_count = 0;

        self.pos_x = vec![0.0; count];
        self.pos_y = vec![0.0; count];
        self.pos_z = vec![0.0; count];
        self.vel_x = vec![0.0; count];
        self.vel_y = vec![0.0; count];
        self.vel_z = vec![0.0; count];
        self.rot_x = vec![0.0; count];
        self.rot_y = vec![0.0; count];
        self.rot_z = vec![0.0; count];
        self.rot_speed_x = vec![0.0; count];
        self.rot_speed_y = vec![0.0; count];
        self.rot_speed_z = vec![0.0; count];
        self.tree_pos_x = vec![0.0; count];
        self.tree_pos_y = vec![0.0; count];
        self.tree_pos_z = vec![0.0; count];
        self.tree_rot_x = vec![0.0; count];
        self.tree_rot_y = vec![0.0; count];
        self.tree_rot_z = vec![0.0; count];
        self.phase = vec![0.0; count];
        self.scale = vec![1.0; count];
        self.brightness = vec![1.0; count];
        self.ground_offset = vec![0.0; count];
        self.fall_delay = vec![0; count];
        self.state = vec![ON_TREE; count];
        self.active_indices = vec![0; count];
        self.activated_indices = vec![0; count];
        self.landed_indices = vec![0; count];
    }

    fn seed_petals(&mut self, seed: u32, blossom_positions: &[f32]) -> u32 {
        let blossom_count = blossom_positions.len() / 3;
        if blossom_count == 0 {
            self.active_count = 0;
            self.activated_count = 0;
            self.landed_count = 0;
            return 0;
        }

        self.rng_state = seed.max(1);
        self.active_count = 0;
        self.activated_count = 0;
        self.landed_count = 0;

        for i in 0..self.petal_count {
            let blossom_index = self.rand_index(blossom_count);
            let base = blossom_index * 3;
            let x = blossom_positions[base] + self.rand_range(-0.24, 0.24);
            let y = blossom_positions[base + 1] + self.rand_range(-0.14, 0.14);
            let z = blossom_positions[base + 2] + self.rand_range(-0.24, 0.24);

            self.tree_pos_x[i] = x;
            self.tree_pos_y[i] = y;
            self.tree_pos_z[i] = z;
            self.pos_x[i] = x;
            self.pos_y[i] = y;
            self.pos_z[i] = z;

            self.tree_rot_x[i] = self.rand_range(-0.4, 0.4);
            self.tree_rot_y[i] = self.rand_range(0.0, PI * 2.0);
            self.tree_rot_z[i] = self.rand_range(-0.4, 0.4);
            self.rot_x[i] = self.tree_rot_x[i];
            self.rot_y[i] = self.tree_rot_y[i];
            self.rot_z[i] = self.tree_rot_z[i];

            self.rot_speed_x[i] = self.rand_range(-0.04, 0.04);
            self.rot_speed_y[i] = self.rand_range(-0.02, 0.02);
            self.rot_speed_z[i] = self.rand_range(-0.04, 0.04);
            self.vel_x[i] = 0.0;
            self.vel_y[i] = 0.0;
            self.vel_z[i] = 0.0;

            self.phase[i] = self.rand_range(0.0, PI * 2.0);
            self.scale[i] = self.rand_range(0.96, 1.54);
            self.brightness[i] = self.rand_range(0.88, 1.12);
            self.ground_offset[i] = self.rand_range(0.0, 0.01);
            self.fall_delay[i] = 180 + self.rand_index(3600) as i32;
            self.state[i] = ON_TREE;
        }

        blossom_count as u32
    }

    fn activate_ready_petals(&mut self, frame_count: u32, time: f32) -> u32 {
        self.activated_count = 0;

        for i in 0..self.petal_count {
            if self.state[i] != ON_TREE || self.fall_delay[i] > frame_count as i32 {
                continue;
            }

            let sway_x = (time * 1.35 + self.phase[i]).sin() * 0.013;
            let sway_y = (time * 1.05 + self.phase[i] * 0.6).cos() * 0.007;
            let sway_z = (time * 1.45 + self.phase[i] * 1.2).cos() * 0.013;

            self.state[i] = FALLING;
            self.pos_x[i] = self.tree_pos_x[i] + sway_x;
            self.pos_y[i] = self.tree_pos_y[i] + sway_y;
            self.pos_z[i] = self.tree_pos_z[i] + sway_z;
            self.rot_x[i] = self.tree_rot_x[i];
            self.rot_y[i] = self.tree_rot_y[i];
            self.rot_z[i] = self.tree_rot_z[i];
            self.vel_x[i] = sway_x * 0.12 + self.rand_range(-0.0012, 0.0012);
            self.vel_y[i] = self.rand_range(-0.005, -0.003);
            self.vel_z[i] = sway_z * 0.12 + self.rand_range(-0.0012, 0.0012);
            self.active_indices[self.active_count] = i as u32;
            self.active_count += 1;
            self.activated_indices[self.activated_count] = i as u32;
            self.activated_count += 1;
        }

        self.activated_count as u32
    }

    #[allow(clippy::too_many_arguments)]
    fn step_falling(
        &mut self,
        dt: f32,
        time: f32,
        wind_strength: f32,
        gravity: f32,
        air_resistance: f32,
        island_radius: f32,
        ground_y: f32,
        ground_y_outer: f32,
    ) -> u32 {
        self.landed_count = 0;

        let dt_scale = (dt * 60.0).max(0.0);
        let drag = air_resistance.powf(dt_scale.max(0.0001));
        let mut write_index = 0;

        for read_index in 0..self.active_count {
            let idx = self.active_indices[read_index] as usize;
            if self.state[idx] != FALLING {
                continue;
            }

            let phase = self.phase[idx];
            let wind = (time * 0.8 + phase).sin() * wind_strength
                + (time * 1.7 + phase * 1.3).sin() * wind_strength * 0.4
                + (time * 0.3 + phase * 0.7).sin() * wind_strength * 0.6;

            self.vel_y[idx] -= gravity * dt_scale;
            self.vel_x[idx] += wind * dt_scale;
            self.vel_z[idx] += (time * 0.6 + phase).cos() * wind_strength * 0.8 * dt_scale;

            self.vel_x[idx] *= drag;
            self.vel_y[idx] *= drag;
            self.vel_z[idx] *= drag;

            self.pos_x[idx] += self.vel_x[idx] * dt_scale;
            self.pos_y[idx] += self.vel_y[idx] * dt_scale;
            self.pos_z[idx] += self.vel_z[idx] * dt_scale;

            self.rot_x[idx] += self.rot_speed_x[idx] * dt_scale;
            self.rot_y[idx] += self.rot_speed_y[idx] * dt_scale;
            self.rot_z[idx] += self.rot_speed_z[idx] * dt_scale;

            let dist =
                (self.pos_x[idx] * self.pos_x[idx] + self.pos_z[idx] * self.pos_z[idx]).sqrt();
            let ground_level = if dist < island_radius {
                ground_y
            } else {
                ground_y_outer
            };

            if self.pos_y[idx] <= ground_level {
                self.state[idx] = ON_GROUND;
                self.pos_y[idx] = ground_level + self.ground_offset[idx];
                self.vel_x[idx] = 0.0;
                self.vel_y[idx] = 0.0;
                self.vel_z[idx] = 0.0;
                self.rot_speed_x[idx] = 0.0;
                self.rot_speed_y[idx] = 0.0;
                self.rot_speed_z[idx] = 0.0;
                self.rot_x[idx] = -PI * 0.5 + self.rand_range(-0.09, 0.09);
                self.rot_y[idx] = self.rand_range(0.0, PI * 2.0);
                self.rot_z[idx] = self.rand_range(-0.06, 0.06);
                self.landed_indices[self.landed_count] = idx as u32;
                self.landed_count += 1;
                continue;
            }

            self.active_indices[write_index] = idx as u32;
            write_index += 1;
        }

        self.active_count = write_index;
        self.landed_count as u32
    }

    fn rand_u32(&mut self) -> u32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 17;
        self.rng_state ^= self.rng_state << 5;
        self.rng_state
    }

    fn rand_f32(&mut self) -> f32 {
        self.rand_u32() as f32 / u32::MAX as f32
    }

    fn rand_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.rand_f32()
    }

    fn rand_index(&mut self, max: usize) -> usize {
        if max <= 1 {
            return 0;
        }
        (self.rand_u32() as usize) % max
    }
}

fn with_sim<R>(f: impl FnOnce(&mut SakuraSim) -> R) -> R {
    SIM.with(|sim| f(&mut sim.borrow_mut()))
}

fn with_sim_ref<R>(f: impl FnOnce(&SakuraSim) -> R) -> R {
    SIM.with(|sim| f(&sim.borrow()))
}

#[no_mangle]
pub extern "C" fn alloc_f32(len: u32) -> *mut f32 {
    let mut data = Vec::<f32>::with_capacity(len as usize);
    let ptr = data.as_mut_ptr();
    std::mem::forget(data);
    ptr
}

#[no_mangle]
pub extern "C" fn free_f32(ptr: *mut f32, len: u32) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        let _ = Vec::<f32>::from_raw_parts(ptr, 0, len as usize);
    }
}

#[no_mangle]
pub extern "C" fn sim_init(count: u32) -> u32 {
    with_sim(|sim| {
        sim.reset(count);
        sim.petal_count as u32
    })
}

#[no_mangle]
pub extern "C" fn sim_seed_petals(seed: u32, blossom_ptr: *const f32, blossom_len: u32) -> u32 {
    if blossom_ptr.is_null() || blossom_len == 0 {
        return 0;
    }

    let blossom_positions = unsafe { slice::from_raw_parts(blossom_ptr, blossom_len as usize) };
    with_sim(|sim| sim.seed_petals(seed, blossom_positions))
}

#[no_mangle]
pub extern "C" fn sim_activate_ready_petals(frame_count: u32, time: f32) -> u32 {
    with_sim(|sim| sim.activate_ready_petals(frame_count, time))
}

#[no_mangle]
pub extern "C" fn sim_step_falling(
    dt: f32,
    time: f32,
    wind_strength: f32,
    gravity: f32,
    air_resistance: f32,
    island_radius: f32,
    ground_y: f32,
    ground_y_outer: f32,
) -> u32 {
    with_sim(|sim| {
        sim.step_falling(
            dt,
            time,
            wind_strength,
            gravity,
            air_resistance,
            island_radius,
            ground_y,
            ground_y_outer,
        )
    })
}

#[no_mangle]
pub extern "C" fn sim_petal_count() -> u32 {
    with_sim_ref(|sim| sim.petal_count as u32)
}

#[no_mangle]
pub extern "C" fn sim_active_count() -> u32 {
    with_sim_ref(|sim| sim.active_count as u32)
}

#[no_mangle]
pub extern "C" fn sim_activated_count() -> u32 {
    with_sim_ref(|sim| sim.activated_count as u32)
}

#[no_mangle]
pub extern "C" fn sim_landed_count() -> u32 {
    with_sim_ref(|sim| sim.landed_count as u32)
}

#[no_mangle]
pub extern "C" fn sim_pos_x_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.pos_x.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_pos_y_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.pos_y.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_pos_z_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.pos_z.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_rot_x_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.rot_x.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_rot_y_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.rot_y.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_rot_z_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.rot_z.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_tree_pos_x_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.tree_pos_x.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_tree_pos_y_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.tree_pos_y.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_tree_pos_z_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.tree_pos_z.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_tree_rot_x_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.tree_rot_x.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_tree_rot_y_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.tree_rot_y.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_tree_rot_z_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.tree_rot_z.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_phase_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.phase.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_scale_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.scale.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_brightness_ptr() -> *const f32 {
    with_sim_ref(|sim| sim.brightness.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_state_ptr() -> *const u8 {
    with_sim_ref(|sim| sim.state.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_active_indices_ptr() -> *const u32 {
    with_sim_ref(|sim| sim.active_indices.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_activated_indices_ptr() -> *const u32 {
    with_sim_ref(|sim| sim.activated_indices.as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_landed_indices_ptr() -> *const u32 {
    with_sim_ref(|sim| sim.landed_indices.as_ptr())
}
