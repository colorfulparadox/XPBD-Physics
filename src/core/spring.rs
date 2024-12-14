#![allow(unused)]

use bevy::prelude::*;

const EPSILON: f32 = 0.0001;

/*
    ---
    SOURCE
    https://www.ryanjuckett.com/damped-springs/
    ---
*/

pub struct SpringPlugin;

impl Plugin for SpringPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Spring>()
            .register_type::<SpringVec>()
            .add_systems(PostUpdate, update_springs);
    }   
}

fn update_springs(
    mut spring_query: Query<&mut Spring, Without<SpringUpdateIgnore>>,
    mut spring_vec_query: Query<(&mut SpringVec, &mut Transform), (Without<SpringUpdateIgnore>, Without<Spring>)>,
    dt: Res<Time>
) {
    for mut spring in spring_query.iter_mut() {
        spring.update(dt.delta_seconds());
    }

    for (mut spring, mut transform) in spring_vec_query.iter_mut() {
        spring.update(dt.delta_seconds());
        transform.translation = spring.position.extend(transform.translation.z);
    }
}

#[derive(Component)]
pub struct SpringUpdateIgnore;

#[derive(Component, Reflect)]
pub struct Spring {
    pub angular_frequency: f32, 
    pub damping_ratio: f32,

    pub target: f32,
    pub position: f32,
    pub velocity: f32,

    pos_coef: f32,
    pos_vel_coef: f32,
    vel_pos_coef: f32,
    vel_coef: f32
}

impl Spring {
    pub fn new(target: f32, angular_freq: f32, damping_ratio: f32) -> Spring {
        let mut new_spring = Spring {
            angular_frequency: 0.,
            damping_ratio: 0.,

            target: target,
            position: 0.,
            velocity: 0.,

            pos_coef: 0.0,
            pos_vel_coef: 0.0,
            vel_pos_coef: 0.0,
            vel_coef: 0.0,
        };

        new_spring.set_angular(angular_freq);
        new_spring.set_damping(damping_ratio);

        return new_spring;
    }

    pub fn set_damping(&mut self, ratio: f32) {
        self.damping_ratio = if ratio < 0.0 {
            0.0
        } else {
            ratio
        };
    }
    
    pub fn set_angular(&mut self, angle: f32) {
        self.angular_frequency = if angle < 0.0 {
            0.0
        } else {
            angle
        };
    }

    pub fn shove(
        &mut self,
        goal: f32,
    ) {
        self.position += goal;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.angular_frequency < EPSILON {
            self.pos_coef = 1.0;
            self.pos_vel_coef = 0.0;
            self.vel_pos_coef = 0.0;
            self.vel_coef = 1.0;
            return;
        }

        if self.damping_ratio > (1.0 + EPSILON) {
            //over-damped
            let za = -self.angular_frequency * self.damping_ratio;
            let zb = self.angular_frequency * f32::sqrt(self.damping_ratio * self.damping_ratio - 1.0);
            let z1 = za - zb;
            let z2 = za + zb;

            let e1 = f32::exp(z1 * delta_time);
            let e2 = f32::exp(z2 * delta_time);

            let inv_two_zb = 1.0 / (2.0 * zb);

            let e1_over_twozb = e1 * inv_two_zb;
            let e2_over_twozb = e2 * inv_two_zb;

            let z1e1_over_twozb = z1 * e1_over_twozb;
            let z2e2_over_twozb = z2*e2_over_twozb;

            self.pos_coef = e1_over_twozb * z2 - z2e2_over_twozb + e2;
            self.pos_vel_coef = -e1_over_twozb + e2_over_twozb;

            self.vel_pos_coef = (z1e1_over_twozb - z2e2_over_twozb + e2) * z2;
            self.vel_coef = -z1e1_over_twozb + z2e2_over_twozb;
        } else if self.damping_ratio < (1.0 - EPSILON) {
            //under-damped
            let omega_zeta = self.angular_frequency * self.damping_ratio;
            let alpha = self.angular_frequency * f32::sqrt(1.0 - (self.damping_ratio * self.damping_ratio));

            let exp_term = f32::exp(-omega_zeta * delta_time);
            let cos_term = f32::cos(alpha * delta_time);
            let sin_term = f32::sin(alpha * delta_time);

            let inv_alpha = 1.0 / alpha;

            let exp_sin = exp_term * sin_term;
            let exp_cos = exp_term * cos_term;
            let expomega_zeta_sin_over_alpha = exp_term * omega_zeta * sin_term * inv_alpha;

            self.pos_coef = exp_cos + expomega_zeta_sin_over_alpha;
            self.pos_vel_coef = exp_sin * inv_alpha;

            self.vel_pos_coef = -exp_sin * alpha - omega_zeta * expomega_zeta_sin_over_alpha;
            self.vel_coef = exp_cos - expomega_zeta_sin_over_alpha;
        } else {
            //critically damped
            let exp_term = f32::exp(-self.angular_frequency * delta_time);
            let time_exp = delta_time * exp_term;
            let time_exp_freq = time_exp * self.angular_frequency;

            self.pos_coef = time_exp_freq + exp_term;
            self.pos_vel_coef = time_exp;

            self.vel_pos_coef = -self.angular_frequency * time_exp_freq;
            self.vel_coef = -time_exp_freq + exp_term;
        }

        //update spring
        let old_pos = self.position - self.target;
        let old_vel = self.velocity;

        self.position = old_pos * self.pos_coef + old_vel * self.pos_vel_coef + self.target;
        self.velocity = old_pos * self.vel_pos_coef + old_vel * self.vel_coef;
    }
}


#[derive(Component, Reflect)]
pub struct SpringVec {
    pub angular_frequency: f32, 
    pub damping_ratio: f32,

    pub target: Vec2,
    pub position: Vec2,
    pub velocity: Vec2,

    pos_coef: f32,
    pos_vel_coef: f32,
    vel_pos_coef: f32,
    vel_coef: f32
}

impl SpringVec {
    pub fn new(target: Vec2, angular_freq: f32, damping_ratio: f32) -> SpringVec {
        let mut new_spring = SpringVec {
            angular_frequency: 0.,
            damping_ratio: 0.,

            target: target,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,

            pos_coef: 0.0,
            pos_vel_coef: 0.0,
            vel_pos_coef: 0.0,
            vel_coef: 0.0,
        };

        new_spring.set_angular(angular_freq);
        new_spring.set_damping(damping_ratio);

        return new_spring;
    }

    pub fn set_damping(&mut self, ratio: f32) {
        self.damping_ratio = if ratio < 0.0 {
            0.0
        } else {
            ratio
        };
    }
    
    pub fn set_angular(&mut self, angle: f32) {
        self.angular_frequency = if angle < 0.0 {
            0.0
        } else {
            angle
        };
    }

    pub fn set_target(&mut self, new_target: Vec2) {
        //self.position += -new_target;
        self.target = new_target;
    }

    pub fn shove(
        &mut self,
        goal: Vec2,
    ) {
        self.position += goal;
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.angular_frequency < EPSILON {
            self.pos_coef = 1.0;
            self.pos_vel_coef = 0.0;
            self.vel_pos_coef = 0.0;
            self.vel_coef = 1.0;
            return;
        }

        if self.damping_ratio > (1.0 + EPSILON) {
            //over-damped
            let za = -self.angular_frequency * self.damping_ratio;
            let zb = self.angular_frequency * f32::sqrt(self.damping_ratio * self.damping_ratio - 1.0);
            let z1 = za - zb;
            let z2 = za + zb;

            let e1 = f32::exp(z1 * delta_time);
            let e2 = f32::exp(z2 * delta_time);

            let inv_two_zb = 1.0 / (2.0 * zb);

            let e1_over_twozb = e1 * inv_two_zb;
            let e2_over_twozb = e2 * inv_two_zb;

            let z1e1_over_twozb = z1 * e1_over_twozb;
            let z2e2_over_twozb = z2 * e2_over_twozb;

            self.pos_coef = e1_over_twozb * z2 - z2e2_over_twozb + e2;
            self.pos_vel_coef = -e1_over_twozb + e2_over_twozb;

            self.vel_pos_coef = (z1e1_over_twozb - z2e2_over_twozb + e2) * z2;
            self.vel_coef = -z1e1_over_twozb + z2e2_over_twozb;
        } else if self.damping_ratio < (1.0 - EPSILON) {
            //under-damped
            let omega_zeta = self.angular_frequency * self.damping_ratio;
            let alpha = self.angular_frequency * f32::sqrt(1.0 - (self.damping_ratio * self.damping_ratio));

            let exp_term = f32::exp(-omega_zeta * delta_time);
            let cos_term = f32::cos(alpha * delta_time);
            let sin_term = f32::sin(alpha * delta_time);

            let inv_alpha = 1.0 / alpha;

            let exp_sin = exp_term * sin_term;
            let exp_cos = exp_term * cos_term;
            let expomega_zeta_sin_over_alpha = exp_term * omega_zeta * sin_term * inv_alpha;

            self.pos_coef = exp_cos + expomega_zeta_sin_over_alpha;
            self.pos_vel_coef = exp_sin * inv_alpha;

            self.vel_pos_coef = -exp_sin * alpha - omega_zeta * expomega_zeta_sin_over_alpha;
            self.vel_coef = exp_cos - expomega_zeta_sin_over_alpha;
        } else {
            //critically damped
            let exp_term = f32::exp(-self.angular_frequency * delta_time);
            let time_exp = delta_time * exp_term;
            let time_exp_freq = time_exp * self.angular_frequency;

            self.pos_coef = time_exp_freq + exp_term;
            self.pos_vel_coef = time_exp;

            self.vel_pos_coef = -self.angular_frequency * time_exp_freq;
            self.vel_coef = -time_exp_freq + exp_term;
        }

        //update spring
        let old_pos = self.position - self.target;
        let old_vel = self.velocity;

        self.position = old_pos * self.pos_coef + old_vel * self.pos_vel_coef + self.target;
        self.velocity = old_pos * self.vel_pos_coef + old_vel * self.vel_coef;
    }
}