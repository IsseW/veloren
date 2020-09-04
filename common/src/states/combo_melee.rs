use crate::{
    comp::{Attacking, CharacterState, EnergySource, StateUpdate},
    states::utils::*,
    sys::character_behavior::{CharacterBehavior, JoinData},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stage {
    /// Specifies which stage the combo attack is in
    pub stage: u32,
    /// Initial damage of stage
    pub base_damage: u32,
    /// Max damage of stage
    pub max_damage: u32,
    /// Damage scaling per combo
    pub damage_increase: u32,
    /// Knockback of stage
    pub knockback: f32,
    /// Range of attack
    pub range: f32,
    /// Angle of attack
    pub angle: f32,
    /// Initial buildup duration of stage (how long until state can deal damage)
    pub base_buildup_duration: Duration,
    /// Initial recover duration of stage (how long until character exits state)
    pub base_recover_duration: Duration,
}

/// A sequence of attacks that can incrementally become faster and more
/// damaging.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    /// Indicates what stage the combo is in
    pub stage: u32,
    /// Indicates number of stages in combo
    pub num_stages: u32,
    /// Number of consecutive strikes
    pub combo: u32,
    /// Data for first stage
    pub stage_data: Vec<Stage>,
    /// Whether state can deal more damage
    pub exhausted: bool,
    /// Initial energy gain per strike
    pub initial_energy_gain: u32,
    /// Max energy gain per strike
    pub max_energy_gain: u32,
    /// Energy gain increase per combo
    pub energy_increase: u32,
    /// Duration for the next stage to be activated
    pub combo_duration: Duration,
    /// Timer for each stage
    pub timer: Duration,
}

impl CharacterBehavior for Data {
    fn behavior(&self, data: &JoinData) -> StateUpdate {
        let mut update = StateUpdate::from(data);

        handle_orientation(data, &mut update, 5.0);
        handle_move(data, &mut update, 0.8);

        let stage_index = (self.stage - 1) as usize;

        if !self.exhausted && self.timer < self.stage_data[stage_index].base_buildup_duration {
            // Build up
            update.character = CharacterState::ComboMelee(Data {
                stage: self.stage,
                num_stages: self.num_stages,
                combo: self.combo,
                stage_data: self.stage_data.clone(),
                exhausted: self.exhausted,
                initial_energy_gain: self.initial_energy_gain,
                max_energy_gain: self.max_energy_gain,
                energy_increase: self.energy_increase,
                combo_duration: self.combo_duration,
                timer: self
                    .timer
                    .checked_add(Duration::from_secs_f32(data.dt.0))
                    .unwrap_or_default(),
            });
        } else if !self.exhausted {
            // Hit attempt
            data.updater.insert(data.entity, Attacking {
                base_healthchange: -((self.stage_data[stage_index].max_damage.min(
                    self.stage_data[stage_index].base_damage
                        + self.combo / self.num_stages
                            * self.stage_data[stage_index].damage_increase,
                )) as i32),
                range: self.stage_data[stage_index].range,
                max_angle: self.stage_data[stage_index].angle.to_radians(),
                applied: false,
                hit_count: 0,
                knockback: self.stage_data[stage_index].knockback,
            });

            update.character = CharacterState::ComboMelee(Data {
                stage: self.stage,
                num_stages: self.num_stages,
                combo: self.combo,
                stage_data: self.stage_data.clone(),
                exhausted: true,
                initial_energy_gain: self.initial_energy_gain,
                max_energy_gain: self.max_energy_gain,
                energy_increase: self.energy_increase,
                combo_duration: self.combo_duration,
                timer: Duration::default(),
            });
        } else if self.timer < self.stage_data[stage_index].base_recover_duration {
            update.character = CharacterState::ComboMelee(Data {
                stage: self.stage,
                num_stages: self.num_stages,
                combo: self.combo,
                stage_data: self.stage_data.clone(),
                exhausted: self.exhausted,
                initial_energy_gain: self.initial_energy_gain,
                max_energy_gain: self.max_energy_gain,
                energy_increase: self.energy_increase,
                combo_duration: self.combo_duration,
                timer: self
                    .timer
                    .checked_add(Duration::from_secs_f32(data.dt.0))
                    .unwrap_or_default(),
            });
        } else if self.timer
            < self.combo_duration + self.stage_data[stage_index].base_recover_duration
        {
            if data.inputs.primary.is_pressed() {
                update.character = CharacterState::ComboMelee(Data {
                    stage: (self.stage % self.num_stages) + 1,
                    num_stages: self.num_stages,
                    combo: self.combo + 1,
                    stage_data: self.stage_data.clone(),
                    exhausted: false,
                    initial_energy_gain: self.initial_energy_gain,
                    max_energy_gain: self.max_energy_gain,
                    energy_increase: self.energy_increase,
                    combo_duration: self.combo_duration,
                    timer: Duration::default(),
                });
            } else {
                update.character = CharacterState::ComboMelee(Data {
                    stage: self.stage,
                    num_stages: self.num_stages,
                    combo: self.combo,
                    stage_data: self.stage_data.clone(),
                    exhausted: self.exhausted,
                    initial_energy_gain: self.initial_energy_gain,
                    max_energy_gain: self.max_energy_gain,
                    energy_increase: self.energy_increase,
                    combo_duration: self.combo_duration,
                    timer: self
                        .timer
                        .checked_add(Duration::from_secs_f32(data.dt.0))
                        .unwrap_or_default(),
                });
            }
        } else {
            // Done
            update.character = CharacterState::Wielding;
            // Make sure attack component is removed
            data.updater.remove::<Attacking>(data.entity);
        }

        // Grant energy on successful hit
        if let Some(attack) = data.attacking {
            if attack.applied && attack.hit_count > 0 {
                let energy = self
                    .max_energy_gain
                    .min(self.initial_energy_gain + self.combo * self.energy_increase)
                    as i32;
                data.updater.remove::<Attacking>(data.entity);
                update.energy.change_by(energy, EnergySource::HitEnemy);
            }
        }

        update
    }
}
