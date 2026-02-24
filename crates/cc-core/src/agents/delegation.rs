//! Task Delegation and Result Aggregation
//!
//! Implements:
//! - TaskDelegator: Splits and delegates tasks to sub-agents
//! - ResultAggregator: Combines results from multiple sub-agents
//! - ParallelExecutor: Executes tasks in parallel with concurrency control

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinSet;
use tracing::{debug, error, warn};

use super::manager::SubAgentManager;
use super::types::{SubAgentId, SubAgentResult, SubAgentTask, TaskId, TaskPriority};
use crate::llm::{Message, ToolDefinition};
use crate::Result;

/// Configuration for task delegation
#[derive(Debug, Clone)]
pub struct DelegationConfig {
    /// Maximum concurrent sub-agent executions
    pub max_concurrency: usize,
    /// Default timeout for delegated tasks
    pub default_timeout_secs: u64,
    /// Maximum iterations for sub-agent loops
    pub default_max_iterations: usize,
    /// Whether to fail fast on first error
    pub fail_fast: bool,
    /// Whether to retry failed tasks
    pub retry_failed: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for DelegationConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 4,
            default_timeout_secs: 120,
            default_max_iterations: 10,
            fail_fast: false,
            retry_failed: true,
            max_retries: 2,
        }
    }
}

/// Task delegator for splitting and distributing tasks
pub struct TaskDelegator {
    manager: Arc<Mutex<SubAgentManager>>,
    config: DelegationConfig,
}

impl TaskDelegator {
    /// Create a new task delegator
    pub fn new(manager: Arc<Mutex<SubAgentManager>>, config: DelegationConfig) -> Self {
        Self { manager, config }
    }

    /// Create with default configuration
    pub fn with_defaults(manager: Arc<Mutex<SubAgentManager>>) -> Self {
        Self::new(manager, DelegationConfig::default())
    }

    /// Delegate a single task to the best available agent
    pub async fn delegate(&self, task: SubAgentTask) -> Result<SubAgentResult> {
        let manager = self.manager.lock().await;
        manager.execute_with_best_agent(task).await
    }

    /// Delegate a task to a specific agent
    pub async fn delegate_to(
        &self,
        agent_id: SubAgentId,
        task: SubAgentTask,
    ) -> Result<SubAgentResult> {
        let manager = self.manager.lock().await;
        manager.execute_with_agent(&agent_id, task).await
    }

    /// Delegate multiple tasks in parallel
    pub async fn delegate_parallel(
        &self,
        tasks: Vec<SubAgentTask>,
    ) -> Result<Vec<SubAgentResult>> {
        let executor = ParallelExecutor::new(self.manager.clone(), self.config.clone());
        executor.execute_all(tasks).await
    }

    /// Split a complex task into subtasks
    pub fn split_task(&self, task: &SubAgentTask, parts: usize) -> Vec<SubAgentTask> {
        if parts <= 1 {
            return vec![task.clone()];
        }

        // Simple splitting strategy: divide context evenly
        let context_per_part = if task.context.is_empty() {
            0
        } else {
            task.context.len().div_ceil(parts)
        };

        (0..parts)
            .map(|i| {
                let start = i * context_per_part;
                let end = std::cmp::min(start + context_per_part, task.context.len());

                SubAgentTask {
                    id: TaskId::default(),
                    instruction: format!("Part {} of {}: {}", i + 1, parts, task.instruction),
                    context: if context_per_part > 0 {
                        task.context[start..end].to_vec()
                    } else {
                        vec![]
                    },
                    available_tools: task.available_tools.clone(),
                    priority: task.priority,
                    max_iterations: task.max_iterations,
                    max_tokens: task.max_tokens / parts as u64,
                    timeout_secs: task.timeout_secs,
                    metadata: task.metadata.clone(),
                }
            })
            .collect()
    }

    /// Create subtasks with different focuses
    pub fn create_parallel_subtasks(
        &self,
        base_instruction: &str,
        focuses: &[&str],
        context: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> Vec<SubAgentTask> {
        focuses
            .iter()
            .map(|focus| {
                SubAgentTask::builder(format!("{}: {}", focus, base_instruction))
                    .context(context.clone())
                    .tools(tools.clone())
                    .build()
            })
            .collect()
    }

    /// Get the configuration
    pub fn config(&self) -> &DelegationConfig {
        &self.config
    }
}

/// Aggregated result from multiple sub-agent executions
#[derive(Debug, Clone)]
pub struct AggregatedResult {
    /// Individual results from each sub-agent
    pub results: Vec<SubAgentResult>,
    /// Combined output text
    pub combined_output: String,
    /// Overall success status
    pub success: bool,
    /// Total input tokens
    pub total_input_tokens: u64,
    /// Total output tokens
    pub total_output_tokens: u64,
    /// Total execution time in milliseconds
    pub total_time_ms: u64,
    /// Number of successful tasks
    pub successful_count: usize,
    /// Number of failed tasks
    pub failed_count: usize,
}

impl AggregatedResult {
    /// Create from a list of results
    pub fn from_results(results: Vec<SubAgentResult>) -> Self {
        let successful_count = results.iter().filter(|r| r.success).count();
        let failed_count = results.len() - successful_count;

        let combined_output = results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.output.as_str())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let total_input_tokens: u64 = results.iter().map(|r| r.input_tokens).sum();
        let total_output_tokens: u64 = results.iter().map(|r| r.output_tokens).sum();
        let total_time_ms: u64 = results.iter().map(|r| r.execution_time_ms).sum();

        Self {
            success: failed_count == 0,
            results,
            combined_output,
            total_input_tokens,
            total_output_tokens,
            total_time_ms,
            successful_count,
            failed_count,
        }
    }

    /// Create an empty result
    pub fn empty() -> Self {
        Self {
            results: vec![],
            combined_output: String::new(),
            success: true,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_time_ms: 0,
            successful_count: 0,
            failed_count: 0,
        }
    }

    /// Get all outputs as an iterator
    pub fn outputs(&self) -> impl Iterator<Item = &str> {
        self.results.iter().map(|r| r.output.as_str())
    }

    /// Get all errors as an iterator
    pub fn errors(&self) -> impl Iterator<Item = &str> {
        self.results
            .iter()
            .filter_map(|r| r.error.as_deref())
    }

    /// Get results by agent
    pub fn by_agent(&self, agent_id: &SubAgentId) -> Vec<&SubAgentResult> {
        self.results
            .iter()
            .filter(|r| &r.agent_id == agent_id)
            .collect()
    }
}

/// Result aggregator for combining sub-agent results
pub struct ResultAggregator {
    /// Strategy for combining results
    strategy: AggregationStrategy,
}

/// Strategy for aggregating results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregationStrategy {
    /// Combine all outputs with separators
    Concatenate,
    /// Only include successful results
    SuccessOnly,
    /// Include summary statistics
    WithSummary,
    /// Custom aggregation logic
    Custom,
}

impl ResultAggregator {
    /// Create a new result aggregator
    pub fn new(strategy: AggregationStrategy) -> Self {
        Self { strategy }
    }

    /// Create with default strategy
    pub fn with_defaults() -> Self {
        Self::new(AggregationStrategy::SuccessOnly)
    }

    /// Aggregate multiple results
    pub fn aggregate(&self, results: Vec<SubAgentResult>) -> AggregatedResult {
        match self.strategy {
            AggregationStrategy::Concatenate => self.concatenate(results),
            AggregationStrategy::SuccessOnly => self.success_only(results),
            AggregationStrategy::WithSummary => self.with_summary(results),
            AggregationStrategy::Custom => self.custom(results),
        }
    }

    fn concatenate(&self, results: Vec<SubAgentResult>) -> AggregatedResult {
        AggregatedResult::from_results(results)
    }

    fn success_only(&self, results: Vec<SubAgentResult>) -> AggregatedResult {
        let filtered: Vec<_> = results.into_iter().filter(|r| r.success).collect();
        AggregatedResult::from_results(filtered)
    }

    fn with_summary(&self, results: Vec<SubAgentResult>) -> AggregatedResult {
        let mut aggregated = AggregatedResult::from_results(results);

        // Add summary header
        let summary = format!(
            "## Execution Summary\n- Tasks completed: {}/{}\n- Total tokens: {} in / {} out\n- Total time: {}ms\n\n",
            aggregated.successful_count,
            aggregated.successful_count + aggregated.failed_count,
            aggregated.total_input_tokens,
            aggregated.total_output_tokens,
            aggregated.total_time_ms
        );

        aggregated.combined_output = format!("{}\n{}", summary, aggregated.combined_output);
        aggregated
    }

    fn custom(&self, results: Vec<SubAgentResult>) -> AggregatedResult {
        // Default to success_only for custom strategy
        // Can be extended with custom logic
        self.success_only(results)
    }

    /// Aggregate results by priority
    pub fn by_priority(&self, results: Vec<SubAgentResult>) -> HashMap<TaskPriority, AggregatedResult> {
        let mut by_priority: HashMap<TaskPriority, Vec<SubAgentResult>> = HashMap::new();

        for result in results {
            // Find the task's priority from the result's task_id
            // Since we don't have direct access, we'll group all under Normal
            by_priority
                .entry(TaskPriority::Normal)
                .or_default()
                .push(result);
        }

        by_priority
            .into_iter()
            .map(|(priority, results)| (priority, self.aggregate(results)))
            .collect()
    }
}

impl Default for ResultAggregator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Parallel executor with concurrency control
pub struct ParallelExecutor {
    manager: Arc<Mutex<SubAgentManager>>,
    config: DelegationConfig,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(manager: Arc<Mutex<SubAgentManager>>, config: DelegationConfig) -> Self {
        Self { manager, config }
    }

    /// Execute multiple tasks in parallel
    pub async fn execute_all(&self, tasks: Vec<SubAgentTask>) -> Result<Vec<SubAgentResult>> {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let results = Arc::new(Mutex::new(Vec::new()));
        let mut join_set = JoinSet::new();

        let start_time = Instant::now();

        for task in tasks {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let manager = self.manager.clone();
            let results = results.clone();
            let fail_fast = self.config.fail_fast;

            join_set.spawn(async move {
                let result = {
                    let m = manager.lock().await;
                    m.execute_with_best_agent(task).await
                };

                drop(permit);

                match result {
                    Ok(r) => {
                        let mut res = results.lock().await;
                        res.push(r);
                    }
                    Err(e) => {
                        if fail_fast {
                            error!("Task failed, aborting due to fail_fast: {}", e);
                            return Err(e);
                        }
                        warn!("Task failed: {}", e);
                    }
                }

                Ok(())
            });
        }

        // Wait for all tasks to complete
        while let Some(res) = join_set.join_next().await {
            if let Ok(Err(e)) = res {
                if self.config.fail_fast {
                    return Err(e);
                }
            }
        }

        let elapsed = start_time.elapsed();
        debug!(
            "Parallel execution completed in {:?}",
            elapsed
        );

        let final_results = results.lock().await.clone();
        Ok(final_results)
    }

    /// Execute tasks with ordered results
    pub async fn execute_ordered(&self, tasks: Vec<SubAgentTask>) -> Result<Vec<SubAgentResult>> {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let results = Arc::new(Mutex::new(vec![None; tasks.len()]));
        let mut join_set = JoinSet::new();

        for (index, task) in tasks.into_iter().enumerate() {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let manager = self.manager.clone();
            let results = results.clone();

            join_set.spawn(async move {
                let result = {
                    let m = manager.lock().await;
                    m.execute_with_best_agent(task).await
                };

                drop(permit);

                if let Ok(r) = result {
                    let mut res = results.lock().await;
                    res[index] = Some(r);
                }
            });
        }

        // Wait for all tasks
        while join_set.join_next().await.is_some() {}

        let final_results = results
            .lock()
            .await
            .drain(..)
            .flatten()
            .collect();

        Ok(final_results)
    }

    /// Execute tasks with retry logic
    pub async fn execute_with_retry(
        &self,
        mut tasks: Vec<SubAgentTask>,
    ) -> Result<Vec<SubAgentResult>> {
        let max_retries = self.config.max_retries;
        let mut all_results = Vec::new();
        let mut retry_queue: VecDeque<(SubAgentTask, u32)> =
            tasks.drain(..).map(|t| (t, 0)).collect();

        while let Some((task, attempts)) = retry_queue.pop_front() {
            let result = self.execute_all(vec![task.clone()]).await?;

            for r in result {
                if r.success || attempts >= max_retries {
                    all_results.push(r);
                } else {
                    warn!(
                        "Retrying task {} (attempt {}/{})",
                        task.id.as_str(),
                        attempts + 1,
                        max_retries
                    );
                    retry_queue.push_back((task.clone(), attempts + 1));
                }
            }
        }

        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::TaskStatus;

    #[test]
    fn test_delegation_config_default() {
        let config = DelegationConfig::default();
        assert_eq!(config.max_concurrency, 4);
        assert_eq!(config.default_timeout_secs, 120);
        assert!(config.retry_failed);
    }

    #[test]
    fn test_aggregated_result_from_results() {
        let task_id = TaskId::new("test");
        let agent_id = SubAgentId::new("agent");

        let results = vec![
            SubAgentResult::success(task_id.clone(), agent_id.clone(), "Output 1", 1, 10, 5, 100),
            SubAgentResult::success(task_id.clone(), agent_id.clone(), "Output 2", 2, 20, 10, 200),
        ];

        let aggregated = AggregatedResult::from_results(results);

        assert!(aggregated.success);
        assert_eq!(aggregated.successful_count, 2);
        assert_eq!(aggregated.failed_count, 0);
        assert_eq!(aggregated.total_input_tokens, 30);
        assert_eq!(aggregated.total_output_tokens, 15);
        assert_eq!(aggregated.total_time_ms, 300);
    }

    #[test]
    fn test_aggregated_result_with_failures() {
        let task_id = TaskId::new("test");
        let agent_id = SubAgentId::new("agent");

        let results = vec![
            SubAgentResult::success(task_id.clone(), agent_id.clone(), "Success", 1, 10, 5, 100),
            SubAgentResult::failure(task_id.clone(), agent_id.clone(), "Failed", TaskStatus::Failed),
        ];

        let aggregated = AggregatedResult::from_results(results);

        assert!(!aggregated.success);
        assert_eq!(aggregated.successful_count, 1);
        assert_eq!(aggregated.failed_count, 1);
    }

    #[test]
    fn test_result_aggregator_success_only() {
        let aggregator = ResultAggregator::new(AggregationStrategy::SuccessOnly);

        let task_id = TaskId::new("test");
        let agent_id = SubAgentId::new("agent");

        let results = vec![
            SubAgentResult::success(task_id.clone(), agent_id.clone(), "OK", 1, 10, 5, 100),
            SubAgentResult::failure(task_id.clone(), agent_id.clone(), "Error", TaskStatus::Failed),
        ];

        let aggregated = aggregator.aggregate(results);

        assert!(aggregated.success);
        assert_eq!(aggregated.successful_count, 1);
    }

    #[test]
    fn test_result_aggregator_with_summary() {
        let aggregator = ResultAggregator::new(AggregationStrategy::WithSummary);

        let task_id = TaskId::new("test");
        let agent_id = SubAgentId::new("agent");

        let results = vec![
            SubAgentResult::success(task_id.clone(), agent_id.clone(), "Output", 1, 10, 5, 100),
        ];

        let aggregated = aggregator.aggregate(results);

        assert!(aggregated.combined_output.contains("## Execution Summary"));
        assert!(aggregated.combined_output.contains("Tasks completed: 1/1"));
    }

    #[test]
    fn test_task_splitter() {
        let manager = Arc::new(Mutex::new(SubAgentManager::new()));
        let delegator = TaskDelegator::with_defaults(manager);

        let task = SubAgentTask::builder("Complex task")
            .max_tokens(4000)
            .build();

        let subtasks = delegator.split_task(&task, 3);

        assert_eq!(subtasks.len(), 3);
        assert!(subtasks[0].instruction.contains("Part 1"));
        assert!(subtasks[1].instruction.contains("Part 2"));
        assert!(subtasks[2].instruction.contains("Part 3"));
    }

    #[test]
    fn test_create_parallel_subtasks() {
        let manager = Arc::new(Mutex::new(SubAgentManager::new()));
        let delegator = TaskDelegator::with_defaults(manager);

        let subtasks = delegator.create_parallel_subtasks(
            "Analyze this",
            &["security", "performance", "readability"],
            vec![],
            vec![],
        );

        assert_eq!(subtasks.len(), 3);
        assert!(subtasks[0].instruction.contains("security"));
        assert!(subtasks[1].instruction.contains("performance"));
        assert!(subtasks[2].instruction.contains("readability"));
    }
}
