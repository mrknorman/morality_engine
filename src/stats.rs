use std::time::Duration;
use bevy::prelude::*;

use crate::dilemma::{lever::LeverState, dilemma::DilemmaOptionConsequences};

pub struct Decision{
    pub time : Duration,
    pub choice : LeverState
}

#[derive(Resource)]
pub struct DilemmaStats{
    decisions : Vec<Decision>,
    result : Option<LeverState>,
    decision_time_available : Duration,
    decision_time_used : Duration,
    num_fatalities : usize,
    num_decisions : usize,
    average_num_decisions_per_second : Option<f64>,
    duration_before_first_decision : Option<Duration>,
    duration_remaining_at_last_decision : Option<Duration>
}

impl Default for DilemmaStats {
    fn default() -> Self {
        Self { 
            result : None,
            decisions: vec![], 
            decision_time_available : Duration::ZERO,
            decision_time_used : Duration::ZERO,
            num_fatalities: 0,
            num_decisions: 0, 
            average_num_decisions_per_second : None,
            duration_before_first_decision: None,
            duration_remaining_at_last_decision: None
        }
    }
}

impl DilemmaStats {
    pub fn update(&mut self, lever : &LeverState, timer : &Timer) {

        self.decisions.push(
            Decision{
                time : timer.elapsed(),
                choice : *lever
            }
        );
        self.num_decisions = self.decisions.len();
        self.decision_time_available = timer.duration();

        if self.duration_before_first_decision.is_none() {
            self.duration_before_first_decision = Some(timer.elapsed());
        }

        self.duration_remaining_at_last_decision = Some(timer.remaining());
    }

    pub fn new(decision_time_available : Duration) -> Self {
        Self { 
            result : None,
            decisions: vec![], 
            decision_time_available,
            decision_time_used : Duration::ZERO,
            num_fatalities: 0,
            num_decisions: 0, 
            average_num_decisions_per_second : None,
            duration_before_first_decision: None,
            duration_remaining_at_last_decision: None
        }
    }

    pub fn finalize(
        &mut self, 
        consequence : &DilemmaOptionConsequences, 
        lever : &LeverState, 
        timer : &Timer
    ) {
        self.result = Some(*lever);
        self.num_fatalities = consequence.total_fatalities;
        self.decision_time_used = timer.elapsed();

        if self.num_decisions > 0 {
            self.average_num_decisions_per_second = Some(
                self.num_decisions as f64 / timer.elapsed().as_secs_f64()
            );
        }
    } 

    pub fn to_string(&self) -> String {
        // Format the final decision (assuming LeverState implements Debug)
        let final_decision_str = match self.result {
            Some(ref decision) => format!("{:?}", decision),
            None => "None".to_string(),
        };

        // Format optional fields
        let avg_decisions_str = self
            .average_num_decisions_per_second
            .map(|avg| format!("{:.2}", avg))
            .unwrap_or_else(|| "N/A".to_string());
        let before_first_str = self
            .duration_before_first_decision
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());
        let remaining_last_str = self
            .duration_remaining_at_last_decision
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());

        // Build the formatted output string
        format!(
            "Final Result: {}\nNumber of Fatalities: {}\nNumber of Lever Pulls: {}\nAverage Pull Rate: {} Hz \nTime Before First Pull: {}\nTime Remaining at Last Pull: {}\nTotal Time Used: {:.2} s / {:.2} s",
            final_decision_str,
            self.num_fatalities,
            self.num_decisions,
            avg_decisions_str,
            before_first_str,
            remaining_last_str,
            self.decision_time_used.as_secs_f64(),
            self.decision_time_available.as_secs_f64(),
        )
    }
}

#[derive(Resource)]
pub struct GameStats{
    num_dilemmas : usize,
    total_fatalities : usize,
    mean_fatalities : f64,
    total_decisions : usize,
    mean_decisions : f64,
    dilemma_stats : Vec<DilemmaStats>
}

impl Default for GameStats {

    fn default() -> Self {
        Self{
            num_dilemmas : 0,
            total_fatalities : 0,
            mean_fatalities : 0.0,
            total_decisions : 0,
            mean_decisions : 0.0,
            dilemma_stats : vec![]
        }
    }
}

impl GameStats{

    fn update(&mut self, new_dilemma_stats : DilemmaStats){

        self.dilemma_stats.push(new_dilemma_stats);

        self.num_dilemmas = self.dilemma_stats.len(); 
        self.total_fatalities = self.dilemma_stats.iter().fold(
            0, |acc, stats: &DilemmaStats| acc + stats.num_fatalities
        );
        self.mean_fatalities = self.total_fatalities as f64 / self.num_dilemmas as f64;
        self.total_decisions = self.dilemma_stats.iter().fold(
            0, |acc, stats: &DilemmaStats| acc + stats.num_decisions
        );
        self.mean_decisions = self.total_decisions as f64 / self.num_dilemmas as f64;
    }

}