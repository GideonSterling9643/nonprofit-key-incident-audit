use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Incident {
    pub incident_id: String,
    pub affected_workflows: Vec<Workflow>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Workflow {
    DonorReceipts,
    VolunteerReminders,
    CampaignReporting,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    ReconcileReceipts,
    RescheduleReminders,
    ReviewCampaignReports,
}

pub fn response_actions(workflows: &[Workflow]) -> Vec<Action> {
    let mut actions = Vec::new();
    if workflows.contains(&Workflow::DonorReceipts) {
        actions.push(Action::ReconcileReceipts);
    }
    if workflows.contains(&Workflow::VolunteerReminders) {
        actions.push(Action::RescheduleReminders);
    }
    if workflows.contains(&Workflow::CampaignReporting) {
        actions.push(Action::ReviewCampaignReports);
    }
    actions
}

#[derive(Serialize)]
pub struct IncidentReport {
    pub incident_id: String,
    pub temporary_key_id: String,
    pub response_actions: Vec<Action>,
    pub log_search: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipts_and_reporting_do_not_trigger_reminder_recovery() {
        let actions = response_actions(&[
            Workflow::CampaignReporting,
            Workflow::DonorReceipts,
            Workflow::DonorReceipts,
        ]);
        assert_eq!(actions, vec![Action::ReconcileReceipts, Action::ReviewCampaignReports]);
    }
}
