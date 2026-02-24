//! Sub-Agent Manager
//!
//! Manages registration, lookup, and selection of sub-agents.
//! Implements capability-based routing for task delegation.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::types::{AgentCapability, SubAgent, SubAgentId, SubAgentTask, SubAgentResult};
use crate::Result;

/// Manager for registered sub-agents
pub struct SubAgentManager {
    /// Registered agents indexed by ID
    agents: HashMap<SubAgentId, Arc<dyn SubAgent>>,
    /// Name to ID mapping for lookup by name
    name_to_id: HashMap<String, SubAgentId>,
    /// Default agent ID for tasks without specific routing
    default_agent_id: Option<SubAgentId>,
}

impl SubAgentManager {
    /// Create a new empty sub-agent manager
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            name_to_id: HashMap::new(),
            default_agent_id: None,
        }
    }

    /// Register a sub-agent
    pub fn register(&mut self, agent: Arc<dyn SubAgent>) {
        let id = agent.id().clone();
        let name = agent.name().to_string();

        info!("Registering sub-agent: {} ({})", name, id.as_str());

        // Store name to ID mapping
        self.name_to_id.insert(name.clone(), id.clone());

        // Store the agent
        self.agents.insert(id.clone(), agent);

        // Set as default if first agent
        if self.agents.len() == 1 {
            self.default_agent_id = Some(id);
        }
    }

    /// Unregister a sub-agent by ID
    pub fn unregister(&mut self, id: &SubAgentId) -> Option<Arc<dyn SubAgent>> {
        let agent = self.agents.remove(id);

        if let Some(ref a) = agent {
            self.name_to_id.remove(a.name());
            if self.default_agent_id.as_ref() == Some(id) {
                self.default_agent_id = self.agents.keys().next().cloned();
            }
        }

        agent
    }

    /// Get a sub-agent by ID
    pub fn get(&self, id: &SubAgentId) -> Option<Arc<dyn SubAgent>> {
        self.agents.get(id).cloned()
    }

    /// Get a sub-agent by name
    pub fn get_by_name(&self, name: &str) -> Option<Arc<dyn SubAgent>> {
        self.name_to_id
            .get(name)
            .and_then(|id| self.agents.get(id).cloned())
    }

    /// Get the default sub-agent
    pub fn get_default(&self) -> Option<Arc<dyn SubAgent>> {
        self.default_agent_id
            .as_ref()
            .and_then(|id| self.agents.get(id).cloned())
    }

    /// Set the default agent by ID
    pub fn set_default(&mut self, id: SubAgentId) -> bool {
        if self.agents.contains_key(&id) {
            self.default_agent_id = Some(id);
            true
        } else {
            warn!("Cannot set default: agent not found: {}", id.as_str());
            false
        }
    }

    /// Find the best agent for a task based on capabilities
    pub fn find_best_agent(&self, task: &SubAgentTask) -> Option<Arc<dyn SubAgent>> {
        let mut best_match: Option<(Arc<dyn SubAgent>, usize)> = None;

        for agent in self.agents.values() {
            if !agent.can_handle(task) {
                continue;
            }

            // Count matching capabilities
            let match_score = agent
                .capabilities()
                .iter()
                .filter(|c| c.matches(&task.instruction))
                .count();

            // Update best match if this agent has more matches
            match &best_match {
                None => best_match = Some((agent.clone(), match_score)),
                Some((_, best_score)) if match_score > *best_score => {
                    best_match = Some((agent.clone(), match_score));
                }
                _ => {}
            }
        }

        // Return best match or default agent
        best_match.map(|(a, _)| a).or_else(|| self.get_default())
    }

    /// Get all registered agents
    pub fn all_agents(&self) -> Vec<Arc<dyn SubAgent>> {
        self.agents.values().cloned().collect()
    }

    /// Get all registered agent IDs
    pub fn agent_ids(&self) -> Vec<SubAgentId> {
        self.agents.keys().cloned().collect()
    }

    /// Get all registered agent names
    pub fn agent_names(&self) -> Vec<&str> {
        self.name_to_id.keys().map(|s| s.as_str()).collect()
    }

    /// Check if an agent is registered
    pub fn contains(&self, id: &SubAgentId) -> bool {
        self.agents.contains_key(id)
    }

    /// Get the number of registered agents
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Check if no agents are registered
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Get all capabilities across all agents
    pub fn all_capabilities(&self) -> Vec<(&SubAgentId, Vec<AgentCapability>)> {
        self.agents
            .iter()
            .map(|(id, agent)| (id, agent.capabilities()))
            .collect()
    }

    /// Execute a task with the best matching agent
    pub async fn execute_with_best_agent(
        &self,
        task: SubAgentTask,
    ) -> Result<SubAgentResult> {
        let agent = self
            .find_best_agent(&task)
            .ok_or_else(|| crate::Error::Other("No agent available for task".to_string()))?;

        debug!(
            "Routing task {} to agent: {}",
            task.id.as_str(),
            agent.name()
        );

        agent.execute(task).await
    }

    /// Execute a task with a specific agent
    pub async fn execute_with_agent(
        &self,
        agent_id: &SubAgentId,
        task: SubAgentTask,
    ) -> Result<SubAgentResult> {
        let agent = self
            .get(agent_id)
            .ok_or_else(|| crate::Error::Other(format!("Agent not found: {}", agent_id.as_str())))?;

        agent.execute(task).await
    }
}

impl Default for SubAgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::TaskId;

    // Mock agent for testing
    struct MockAgent {
        id: SubAgentId,
        name: String,
        capabilities: Vec<AgentCapability>,
    }

    impl MockAgent {
        fn new(name: &str, capabilities: Vec<AgentCapability>) -> Self {
            Self {
                id: SubAgentId::new(name),
                name: name.to_string(),
                capabilities,
            }
        }
    }

    #[async_trait::async_trait]
    impl SubAgent for MockAgent {
        fn id(&self) -> &SubAgentId {
            &self.id
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock agent for testing"
        }

        fn capabilities(&self) -> Vec<AgentCapability> {
            self.capabilities.clone()
        }

        async fn execute(&self, _task: SubAgentTask) -> Result<SubAgentResult> {
            Ok(SubAgentResult::success(
                TaskId::default(),
                self.id.clone(),
                "Mock result",
                1,
                10,
                10,
                100,
            ))
        }
    }

    #[test]
    fn test_manager_register() {
        let mut manager = SubAgentManager::new();
        let agent = Arc::new(MockAgent::new(
            "test",
            vec![AgentCapability::new("test", "Test capability")],
        ));

        manager.register(agent);

        assert_eq!(manager.len(), 1);
        assert!(manager.get_by_name("test").is_some());
    }

    #[test]
    fn test_manager_unregister() {
        let mut manager = SubAgentManager::new();
        let agent = Arc::new(MockAgent::new(
            "test",
            vec![AgentCapability::new("test", "Test capability")],
        ));

        manager.register(agent);
        let id = SubAgentId::new("test");

        let removed = manager.unregister(&id);
        assert!(removed.is_some());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_manager_find_best_agent() {
        let mut manager = SubAgentManager::new();

        let code_cap =
            AgentCapability::new("code", "Code analysis").with_keywords(vec!["code".into(), "analyze".into()]);
        let review_cap =
            AgentCapability::new("review", "Code review").with_keywords(vec!["review".into()]);

        manager.register(Arc::new(MockAgent::new("coder", vec![code_cap])));
        manager.register(Arc::new(MockAgent::new("reviewer", vec![review_cap])));

        // code_task has "code" and "analyze" keywords -> coder matches both (2 matches)
        let code_task = SubAgentTask::new("Please analyze this code for bugs");
        // review_task has "review" keyword -> reviewer matches (1 match)
        // review_task also has "code" keyword but reviewer doesn't have "code"
        let review_task = SubAgentTask::new("Please perform a review of this document");

        let best_for_code = manager.find_best_agent(&code_task);
        assert!(best_for_code.is_some());
        assert_eq!(best_for_code.unwrap().name(), "coder");

        let best_for_review = manager.find_best_agent(&review_task);
        assert!(best_for_review.is_some());
        assert_eq!(best_for_review.unwrap().name(), "reviewer");
    }

    #[test]
    fn test_manager_default_agent() {
        let mut manager = SubAgentManager::new();

        let agent1 = Arc::new(MockAgent::new("agent1", vec![]));
        let agent2 = Arc::new(MockAgent::new("agent2", vec![]));

        manager.register(agent1);
        manager.register(agent2);

        // First registered agent should be default
        let default = manager.get_default();
        assert!(default.is_some());
        assert_eq!(default.unwrap().name(), "agent1");

        // Change default
        manager.set_default(SubAgentId::new("agent2"));
        let new_default = manager.get_default();
        assert_eq!(new_default.unwrap().name(), "agent2");
    }

    #[tokio::test]
    async fn test_manager_execute_with_best_agent() {
        let mut manager = SubAgentManager::new();

        let cap = AgentCapability::new("test", "Test").with_keywords(vec!["test".into()]);
        manager.register(Arc::new(MockAgent::new("tester", vec![cap])));

        let task = SubAgentTask::new("This is a test task");
        let result = manager.execute_with_best_agent(task).await;

        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }
}
