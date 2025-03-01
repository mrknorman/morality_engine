use std::time::Duration;
use bevy::prelude::*;

use crate::dilemma::{lever::LeverState, dilemma::DilemmaOptionConsequences};

#[derive(Clone)]
pub struct Decision{
    pub time : Duration,
    pub choice : LeverState
}

#[derive(Resource, Clone)]
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
            .map(|avg| format!("{:.2} Hz", avg))
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
            "Final Decision: {}\nNumber of Fatalities: {}\nNumber of Lever Pulls: {}\nAverage Pull Rate: {}\nTime Before First Pull: {}\nTime Remaining at Last Pull: {}\nTotal Time Used: {:.2} s / {:.2} s",
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

    fn reset(&mut self) {
        self.decisions.clear();
        self.result = None;
        self.num_fatalities = 0;
        self.num_decisions = 0;
        self.average_num_decisions_per_second = None;
        self.duration_before_first_decision = None;
        self.duration_remaining_at_last_decision = None;
    }
}

#[derive(Resource)]
pub struct GameStats{
    pub num_dilemmas : usize,
    pub total_fatalities : usize,
    pub mean_fatalities : f64,
    pub total_decisions : usize,
    pub mean_decisions : f64,
    pub dilemma_stats : Vec<DilemmaStats>,
    // New fields for aggregated timing statistics:
    pub overall_avg_pull_rate: Option<f64>,
    pub overall_avg_first_pull_time: Option<Duration>,
    pub overall_avg_time_remaining: Option<Duration>,
}

impl Default for GameStats {
    fn default() -> Self {
        Self{
            num_dilemmas : 0,
            total_fatalities : 0,
            mean_fatalities : 0.0,
            total_decisions : 0,
            mean_decisions : 0.0,
            dilemma_stats : vec![],
            overall_avg_pull_rate: None,
            overall_avg_first_pull_time: None,
            overall_avg_time_remaining: None,
        }
    }
}

impl GameStats{

    pub fn update(&mut self, new_dilemma_stats : DilemmaStats){
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

        // Calculate overall average pull rate across dilemmas
        let pull_rates: Vec<f64> = self.dilemma_stats.iter()
            .filter_map(|ds| ds.average_num_decisions_per_second)
            .collect();
        self.overall_avg_pull_rate = if !pull_rates.is_empty() {
            Some(pull_rates.iter().sum::<f64>() / pull_rates.len() as f64)
        } else {
            None
        };

        // Calculate overall average time before the first pull
        let first_pull_times: Vec<f64> = self.dilemma_stats.iter()
            .filter_map(|ds| ds.duration_before_first_decision)
            .map(|d| d.as_secs_f64())
            .collect();
        self.overall_avg_first_pull_time = if !first_pull_times.is_empty() {
            Some(Duration::from_secs_f64(first_pull_times.iter().sum::<f64>() / first_pull_times.len() as f64))
        } else {
            None
        };

        // Calculate overall average time remaining at the last pull
        let remaining_times: Vec<f64> = self.dilemma_stats.iter()
            .filter_map(|ds| ds.duration_remaining_at_last_decision)
            .map(|d| d.as_secs_f64())
            .collect();
        self.overall_avg_time_remaining = if !remaining_times.is_empty() {
            Some(Duration::from_secs_f64(remaining_times.iter().sum::<f64>() / remaining_times.len() as f64))
        } else {
            None
        };
    }

    
	pub fn update_stats(
		mut stats: ResMut<GameStats>,
		mut dilemma_stats: ResMut<DilemmaStats>,
	) {
		stats.update(dilemma_stats.clone());
        dilemma_stats.reset();
	}

    pub fn to_string(&self) -> String {
        // Format the numeric values
        let mean_fatalities_str = format!("{:.2}", self.mean_fatalities);
        let mean_decisions_str = format!("{:.2}", self.mean_decisions);
        let avg_pull_rate_str = self.overall_avg_pull_rate
            .map(|r| format!("{:.2} Hz", r))
            .unwrap_or_else(|| "N/A".to_string());
        let avg_first_pull_time_str = self.overall_avg_first_pull_time
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());
        let avg_time_remaining_str = self.overall_avg_time_remaining
            .map(|d| format!("{:.2} s", d.as_secs_f64()))
            .unwrap_or_else(|| "N/A".to_string());

        // Build the formatted output string
        format!(
            "Total Dilemmas: {}\nTotal Fatalities: {}\nAverage Fatalities per Dilemma: {}\nTotal Lever Pulls: {}\nAverage Pulls Per Dilemma: {}\nAverage Pull Rate: {}\nAverage Time Before First Pull: {}\nAverage Time Remaining at Last Pull: {}",
            self.num_dilemmas,
            self.total_fatalities,
            mean_fatalities_str,
            self.total_decisions,
            mean_decisions_str,
            avg_pull_rate_str,
            avg_first_pull_time_str,
            avg_time_remaining_str,
        )
    }
}