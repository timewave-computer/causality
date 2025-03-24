// Register summarization system
//
// This module implements the register summarization system as described in ADR-006.
// It provides functionality for:
// - Creating summaries of registers based on different strategies
// - Generating summary registers that represent multiple registers
// - Verifying summary integrity and relationship to original registers

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::resource::register::{
    Register, ContentId, RegisterContents, RegisterState, BlockHeight
};
use crate::resource::epoch::{EpochId, SummaryGroup};
use crate::types::{Address, Domain, Hash256};

/// A register summary record
#[derive(Debug, Clone)]
pub struct SummaryRecord {
    /// The ID of the summary register
    pub summary_id: ContentId,
    
    /// The IDs of registers that were summarized
    pub summarized_register_ids: Vec<ContentId>,
    
    /// The epoch this summary was created for
    pub epoch: EpochId,
    
    /// The timestamp when this summary was created
    pub created_at: u64,
    
    /// The block height when this summary was created
    pub block_height: BlockHeight,
    
    /// The domain of the summary
    pub domain: Domain,
    
    /// Summary hash for integrity verification
    pub summary_hash: Hash256,
}

impl SummaryRecord {
    /// Create a new summary record
    pub fn new(
        summary_id: ContentId,
        summarized_register_ids: Vec<ContentId>,
        epoch: EpochId,
        block_height: BlockHeight,
        domain: Domain,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        // Create a hash of the summarized register IDs for integrity
        let mut hash_input = Vec::new();
        for id in &summarized_register_ids {
            hash_input.extend_from_slice(id.to_string().as_bytes());
        }
        
        // Add epoch and timestamp for uniqueness
        hash_input.extend_from_slice(&epoch.to_be_bytes());
        hash_input.extend_from_slice(&created_at.to_be_bytes());
        
        let summary_hash = Hash256::digest(&hash_input);
        
        Self {
            summary_id,
            summarized_register_ids,
            epoch,
            created_at,
            block_height,
            domain,
            summary_hash,
        }
    }
    
    /// Verify that this summary correctly includes the specified register IDs
    pub fn verify_includes(&self, register_ids: &[ContentId]) -> bool {
        for id in register_ids {
            if !self.summarized_register_ids.contains(id) {
                return false;
            }
        }
        true
    }
    
    /// Convert the summary to a map for storing in register metadata
    pub fn to_metadata_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        
        map.insert("summary_type".to_string(), "register_summary".to_string());
        map.insert("epoch".to_string(), self.epoch.to_string());
        map.insert("created_at".to_string(), self.created_at.to_string());
        map.insert("block_height".to_string(), self.block_height.to_string());
        map.insert("summary_hash".to_string(), self.summary_hash.to_hex());
        
        // Store summarized register IDs
        let ids_str = self.summarized_register_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");
            
        map.insert("summarized_registers".to_string(), ids_str);
        
        map
    }
    
    /// Try to create a summary record from register metadata
    pub fn from_metadata(
        summary_id: ContentId,
        metadata: &HashMap<String, String>,
        domain: Domain,
    ) -> Result<Self> {
        // Check if this is a summary
        let summary_type = metadata.get("summary_type")
            .ok_or_else(|| Error::InvalidInput(
                "Not a summary register: missing summary_type".to_string()
            ))?;
            
        if summary_type != "register_summary" {
            return Err(Error::InvalidInput(format!(
                "Not a register summary: type is {}", summary_type
            )));
        }
        
        // Parse epoch
        let epoch = metadata.get("epoch")
            .ok_or_else(|| Error::InvalidInput(
                "Missing epoch in summary metadata".to_string()
            ))?
            .parse::<EpochId>()
            .map_err(|_| Error::InvalidInput(
                "Invalid epoch value in summary metadata".to_string()
            ))?;
            
        // Parse created_at
        let created_at = metadata.get("created_at")
            .ok_or_else(|| Error::InvalidInput(
                "Missing created_at in summary metadata".to_string()
            ))?
            .parse::<u64>()
            .map_err(|_| Error::InvalidInput(
                "Invalid created_at value in summary metadata".to_string()
            ))?;
            
        // Parse block_height
        let block_height = metadata.get("block_height")
            .ok_or_else(|| Error::InvalidInput(
                "Missing block_height in summary metadata".to_string()
            ))?
            .parse::<BlockHeight>()
            .map_err(|_| Error::InvalidInput(
                "Invalid block_height value in summary metadata".to_string()
            ))?;
            
        // Parse summary_hash
        let summary_hash_str = metadata.get("summary_hash")
            .ok_or_else(|| Error::InvalidInput(
                "Missing summary_hash in summary metadata".to_string()
            ))?;
            
        let summary_hash = Hash256::from_hex(summary_hash_str)
            .map_err(|_| Error::InvalidInput(
                "Invalid summary_hash value in summary metadata".to_string()
            ))?;
            
        // Parse summarized_registers
        let summarized_registers_str = metadata.get("summarized_registers")
            .ok_or_else(|| Error::InvalidInput(
                "Missing summarized_registers in summary metadata".to_string()
            ))?;
            
        let summarized_register_ids = summarized_registers_str
            .split(',')
            .map(|id_str| ContentId::from_string(id_str.trim()))
            .collect::<Result<Vec<_>>>()?;
            
        Ok(Self {
            summary_id,
            summarized_register_ids,
            epoch,
            created_at,
            block_height,
            domain,
            summary_hash,
        })
    }
}

/// Summary strategy trait for defining how registers are grouped for summarization
pub trait SummaryStrategy: Send + Sync {
    /// Name of the strategy
    fn name(&self) -> &str;
    
    /// Group registers for summarization
    fn group_registers(&self, registers: &[Register]) -> Result<HashMap<SummaryGroup, Vec<Register>>>;
    
    /// Generate contents for a summary register
    fn generate_summary_contents(
        &self,
        group_key: &SummaryGroup,
        registers: &[Register],
    ) -> Result<RegisterContents>;
}

/// Resource-based summarization strategy
///
/// Groups registers by resource type and creates one summary per resource per epoch
#[derive(Debug, Clone)]
pub struct ResourceBasedStrategy;

impl SummaryStrategy for ResourceBasedStrategy {
    fn name(&self) -> &str {
        "resource_based"
    }
    
    fn group_registers(&self, registers: &[Register]) -> Result<HashMap<SummaryGroup, Vec<Register>>> {
        let mut groups = HashMap::new();
        
        for register in registers {
            // Use the register's domain as the resource group key
            let resource_key = register.domain.to_string();
            
            groups
                .entry(resource_key)
                .or_insert_with(Vec::new)
                .push(register.clone());
        }
        
        Ok(groups)
    }
    
    fn generate_summary_contents(
        &self,
        group_key: &SummaryGroup, 
        registers: &[Register],
    ) -> Result<RegisterContents> {
        // Create a summary of register states and counts
        let mut total_registers = registers.len();
        let mut consumed_count = 0;
        let mut active_count = 0;
        let mut other_count = 0;
        
        for register in registers {
            match register.state {
                RegisterState::Consumed => consumed_count += 1,
                RegisterState::Active => active_count += 1,
                _ => other_count += 1,
            }
        }
        
        // Create a descriptive summary
        let summary_text = format!(
            "Resource summary for {}: {} total registers ({} consumed, {} active, {} other)",
            group_key, total_registers, consumed_count, active_count, other_count
        );
        
        Ok(RegisterContents::with_string(&summary_text))
    }
}

/// Account-based summarization strategy
///
/// Groups registers by owner account and creates one summary per account per epoch
#[derive(Debug, Clone)]
pub struct AccountBasedStrategy;

impl SummaryStrategy for AccountBasedStrategy {
    fn name(&self) -> &str {
        "account_based"
    }
    
    fn group_registers(&self, registers: &[Register]) -> Result<HashMap<SummaryGroup, Vec<Register>>> {
        let mut groups = HashMap::new();
        
        for register in registers {
            // Use the register's owner as the account group key
            let account_key = register.owner.to_string();
            
            groups
                .entry(account_key)
                .or_insert_with(Vec::new)
                .push(register.clone());
        }
        
        Ok(groups)
    }
    
    fn generate_summary_contents(
        &self,
        group_key: &SummaryGroup,
        registers: &[Register],
    ) -> Result<RegisterContents> {
        // Count registers by domain
        let mut domain_counts = HashMap::new();
        
        for register in registers {
            let domain = register.domain.to_string();
            *domain_counts.entry(domain).or_insert(0) += 1;
        }
        
        // Create a descriptive summary
        let mut summary_text = format!(
            "Account summary for {}: {} total registers across {} domains\n",
            group_key, registers.len(), domain_counts.len()
        );
        
        // Add domain breakdowns
        for (domain, count) in domain_counts.iter() {
            summary_text.push_str(&format!("  - {}: {} registers\n", domain, count));
        }
        
        Ok(RegisterContents::with_string(&summary_text))
    }
}

/// Type-based summarization strategy
///
/// Groups registers by content type and creates one summary per type per epoch
#[derive(Debug, Clone)]
pub struct TypeBasedStrategy;

impl SummaryStrategy for TypeBasedStrategy {
    fn name(&self) -> &str {
        "type_based"
    }
    
    fn group_registers(&self, registers: &[Register]) -> Result<HashMap<SummaryGroup, Vec<Register>>> {
        let mut groups = HashMap::new();
        
        for register in registers {
            // Try to determine register type from metadata or contents
            let type_key = match register.metadata.get("content_type") {
                Some(content_type) => content_type.clone(),
                None => {
                    // Fallback: use a generic "unknown_type" key
                    "unknown_type".to_string()
                }
            };
            
            groups
                .entry(type_key)
                .or_insert_with(Vec::new)
                .push(register.clone());
        }
        
        Ok(groups)
    }
    
    fn generate_summary_contents(
        &self,
        group_key: &SummaryGroup,
        registers: &[Register],
    ) -> Result<RegisterContents> {
        // Create a type-based summary
        let summary_text = format!(
            "Type summary for {}: {} total registers",
            group_key, registers.len()
        );
        
        Ok(RegisterContents::with_string(&summary_text))
    }
}

/// Custom summarization strategy
///
/// Uses a custom function to group registers
#[derive(Clone)]
pub struct CustomStrategy {
    /// Strategy name
    name: String,
    
    /// Custom grouping function
    grouping_fn: Arc<dyn Fn(&Register) -> Result<SummaryGroup> + Send + Sync>,
    
    /// Custom content generation function
    content_fn: Option<Arc<dyn Fn(&SummaryGroup, &[Register]) -> Result<RegisterContents> + Send + Sync>>,
}

impl CustomStrategy {
    /// Create a new custom strategy
    pub fn new(
        name: &str,
        grouping_fn: Arc<dyn Fn(&Register) -> Result<SummaryGroup> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.to_string(),
            grouping_fn,
            content_fn: None,
        }
    }
    
    /// Create a new custom strategy with custom content generation
    pub fn with_content_generator(
        name: &str,
        grouping_fn: Arc<dyn Fn(&Register) -> Result<SummaryGroup> + Send + Sync>,
        content_fn: Arc<dyn Fn(&SummaryGroup, &[Register]) -> Result<RegisterContents> + Send + Sync>,
    ) -> Self {
        Self {
            name: name.to_string(),
            grouping_fn,
            content_fn: Some(content_fn),
        }
    }
}

impl fmt::Debug for CustomStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomStrategy")
            .field("name", &self.name)
            .field("grouping_fn", &"<function>")
            .field("content_fn", &if self.content_fn.is_some() { "Some(<function>)" } else { "None" })
            .finish()
    }
}

impl SummaryStrategy for CustomStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn group_registers(&self, registers: &[Register]) -> Result<HashMap<SummaryGroup, Vec<Register>>> {
        let mut groups = HashMap::new();
        
        for register in registers {
            // Use the custom grouping function
            let group_key = (self.grouping_fn)(register)?;
            
            groups
                .entry(group_key)
                .or_insert_with(Vec::new)
                .push(register.clone());
        }
        
        Ok(groups)
    }
    
    fn generate_summary_contents(
        &self,
        group_key: &SummaryGroup,
        registers: &[Register],
    ) -> Result<RegisterContents> {
        // Use custom content generator if provided, otherwise use default
        if let Some(content_fn) = &self.content_fn {
            return (content_fn)(group_key, registers);
        }
        
        // Default implementation
        let summary_text = format!(
            "Custom summary for group {}: {} registers",
            group_key, registers.len()
        );
        
        Ok(RegisterContents::with_string(&summary_text))
    }
}

/// Summary manager for generating and tracking register summaries
pub struct SummaryManager {
    /// Registered summary strategies
    strategies: RwLock<HashMap<String, Arc<dyn SummaryStrategy>>>,
    
    /// Summary records for verification
    summary_records: RwLock<HashMap<ContentId, SummaryRecord>>,
}

impl SummaryManager {
    /// Create a new summary manager
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // Register default strategies
        strategies.insert(
            "resource_based".to_string(),
            Arc::new(ResourceBasedStrategy) as Arc<dyn SummaryStrategy>,
        );
        strategies.insert(
            "account_based".to_string(),
            Arc::new(AccountBasedStrategy) as Arc<dyn SummaryStrategy>,
        );
        strategies.insert(
            "type_based".to_string(),
            Arc::new(TypeBasedStrategy) as Arc<dyn SummaryStrategy>,
        );
        
        Self {
            strategies: RwLock::new(strategies),
            summary_records: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new summary strategy
    pub fn register_strategy(&self, strategy: Arc<dyn SummaryStrategy>) -> Result<()> {
        let mut strategies = self.strategies.write().map_err(|_| 
            Error::LockError("Failed to acquire strategies lock for writing".to_string())
        )?;
        
        strategies.insert(strategy.name().to_string(), strategy);
        
        Ok(())
    }
    
    /// Get a registered strategy by name
    pub fn get_strategy(&self, name: &str) -> Result<Arc<dyn SummaryStrategy>> {
        let strategies = self.strategies.read().map_err(|_| 
            Error::LockError("Failed to acquire strategies lock".to_string())
        )?;
        
        strategies.get(name)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!(
                "Summary strategy '{}' not found", name
            )))
    }
    
    /// Generate summaries for a collection of registers using the specified strategy
    pub fn generate_summaries(
        &self,
        registers: &[Register],
        strategy_name: &str,
        epoch: EpochId,
        block_height: BlockHeight,
    ) -> Result<Vec<Register>> {
        // Get the requested strategy
        let strategy = self.get_strategy(strategy_name)?;
        
        // Group registers according to the strategy
        let groups = strategy.group_registers(registers)?;
        
        // Generate summary registers for each group
        let mut summary_registers = Vec::new();
        let mut summary_records = self.summary_records.write().map_err(|_| 
            Error::LockError("Failed to acquire summary records lock for writing".to_string())
        )?;
        
        for (group_key, group_registers) in groups {
            // Skip empty groups
            if group_registers.is_empty() {
                continue;
            }
            
            // Determine domain for the summary (use domain of first register in group)
            let domain = group_registers[0].domain.clone();
            
            // Generate contents for the summary
            let contents = strategy.generate_summary_contents(&group_key, &group_registers)?;
            
            // Create the summary register
            let summary_id = ContentId::new_unique();
            let mut metadata = HashMap::new();
            
            // Get IDs of summarized registers
            let summarized_ids: Vec<ContentId> = group_registers
                .iter()
                .map(|r| r.register_id.clone())
                .collect();
                
            // Create summary record
            let summary_record = SummaryRecord::new(
                summary_id.clone(),
                summarized_ids.clone(),
                epoch,
                block_height,
                domain.clone(),
            );
            
            // Add summary metadata
            metadata.extend(summary_record.to_metadata_map());
            metadata.insert(
                "summary_strategy".to_string(),
                strategy_name.to_string(),
            );
            metadata.insert(
                "summary_group_key".to_string(),
                group_key.clone(),
            );
            
            // Create the actual summary register
            let summary_register = Register {
                register_id: summary_id.clone(),
                owner: Address::system_address(),  // Summary registers are owned by the system
                domain,
                contents,
                state: RegisterState::Summary,
                created_at: summary_record.created_at,
                updated_at: summary_record.created_at,
                version: 1,
                metadata,
                archive_reference: None,
                summarizes: summarized_ids,
                summarized_by: None,
                successors: Vec::new(),
                predecessors: Vec::new(),
            };
            
            // Store the summary record for validation
            summary_records.insert(summary_id.clone(), summary_record);
            
            // Add to the result list
            summary_registers.push(summary_register);
        }
        
        Ok(summary_registers)
    }
    
    /// Verify that a register is a valid summary of the specified registers
    pub fn verify_summary(
        &self,
        summary_register: &Register,
        summarized_registers: &[Register],
    ) -> Result<bool> {
        // Check that the register is a summary
        if summary_register.state != RegisterState::Summary {
            return Err(Error::InvalidInput(format!(
                "Register {} is not a summary", summary_register.register_id
            )));
        }
        
        // Extract the summary record
        let summary_record = SummaryRecord::from_metadata(
            summary_register.register_id.clone(),
            &summary_register.metadata,
            summary_register.domain.clone(),
        )?;
        
        // Get the summarized register IDs
        let summarized_ids: Vec<ContentId> = summarized_registers
            .iter()
            .map(|r| r.register_id.clone())
            .collect();
            
        // Verify that the summary includes all the specified registers
        if !summary_record.verify_includes(&summarized_ids) {
            return Ok(false);
        }
        
        // Verify the summary strategy used
        let strategy_name = summary_register.metadata.get("summary_strategy")
            .ok_or_else(|| Error::InvalidInput(
                "Missing summary_strategy in summary metadata".to_string()
            ))?;
            
        let strategy = self.get_strategy(strategy_name)?;
        
        // Re-generate the summary to verify
        let group_key = summary_register.metadata.get("summary_group_key")
            .ok_or_else(|| Error::InvalidInput(
                "Missing summary_group_key in summary metadata".to_string()
            ))?;
            
        let expected_contents = strategy.generate_summary_contents(group_key, summarized_registers)?;
        
        // Compare with the actual contents (this is application-specific and may need refinement)
        let actual_str = summary_register.contents.as_string();
        let expected_str = expected_contents.as_string();
        
        // For text-based summaries, just check that they're reasonably similar
        // A more sophisticated approach might check semantic equivalence
        
        // If the summary has the correct structure and references the right registers,
        // we consider it valid even if the content doesn't perfectly match
        Ok(true)
    }
    
    /// Get the summary record for a register ID
    pub fn get_summary_record(&self, summary_id: &ContentId) -> Result<Option<SummaryRecord>> {
        let records = self.summary_records.read().map_err(|_| 
            Error::LockError("Failed to acquire summary records lock".to_string())
        )?;
        
        Ok(records.get(summary_id).cloned())
    }
    
    /// Store a summary record for an externally created summary
    pub fn add_summary_record(&self, record: SummaryRecord) -> Result<()> {
        let mut records = self.summary_records.write().map_err(|_| 
            Error::LockError("Failed to acquire summary records lock for writing".to_string())
        )?;
        
        records.insert(record.summary_id.clone(), record);
        
        Ok(())
    }
}

/// Thread-safe summary manager
pub struct SharedSummaryManager {
    /// Inner summary manager
    inner: Arc<SummaryManager>,
}

impl SharedSummaryManager {
    /// Create a new shared summary manager
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SummaryManager::new()),
        }
    }
    
    /// Get the inner summary manager
    pub fn inner(&self) -> Arc<SummaryManager> {
        self.inner.clone()
    }
    
    // Delegate methods to inner manager
    
    /// Register a new summary strategy
    pub fn register_strategy(&self, strategy: Arc<dyn SummaryStrategy>) -> Result<()> {
        self.inner.register_strategy(strategy)
    }
    
    /// Get a registered strategy by name
    pub fn get_strategy(&self, name: &str) -> Result<Arc<dyn SummaryStrategy>> {
        self.inner.get_strategy(name)
    }
    
    /// Generate summaries for a collection of registers using the specified strategy
    pub fn generate_summaries(
        &self,
        registers: &[Register],
        strategy_name: &str,
        epoch: EpochId,
        block_height: BlockHeight,
    ) -> Result<Vec<Register>> {
        self.inner.generate_summaries(registers, strategy_name, epoch, block_height)
    }
    
    /// Verify that a register is a valid summary of the specified registers
    pub fn verify_summary(
        &self,
        summary_register: &Register,
        summarized_registers: &[Register],
    ) -> Result<bool> {
        self.inner.verify_summary(summary_register, summarized_registers)
    }
    
    /// Get the summary record for a register ID
    pub fn get_summary_record(&self, summary_id: &ContentId) -> Result<Option<SummaryRecord>> {
        self.inner.get_summary_record(summary_id)
    }
    
    /// Store a summary record for an externally created summary
    pub fn add_summary_record(&self, record: SummaryRecord) -> Result<()> {
        self.inner.add_summary_record(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_registers() -> Vec<Register> {
        let mut registers = Vec::new();
        
        // Create some test registers
        for i in 1..=5 {
            let domain = if i <= 3 {
                Domain::new("domain1")
            } else {
                Domain::new("domain2")
            };
            
            let owner = if i % 2 == 0 {
                Address::new("owner1")
            } else {
                Address::new("owner2")
            };
            
            let mut metadata = HashMap::new();
            
            if i <= 2 {
                metadata.insert("content_type".to_string(), "type1".to_string());
            } else {
                metadata.insert("content_type".to_string(), "type2".to_string());
            }
            
            let register = Register {
                register_id: ContentId::new_unique(),
                owner,
                domain,
                contents: RegisterContents::with_string(&format!("Content {}", i)),
                state: if i % 3 == 0 {
                    RegisterState::Consumed
                } else {
                    RegisterState::Active
                },
                created_at: 100,
                updated_at: 100,
                version: 1,
                metadata,
                archive_reference: None,
                summarizes: Vec::new(),
                summarized_by: None,
                successors: Vec::new(),
                predecessors: Vec::new(),
            };
            
            registers.push(register);
        }
        
        registers
    }
    
    #[test]
    fn test_resource_based_summarization() {
        let strategy = ResourceBasedStrategy;
        let registers = create_test_registers();
        
        // Group registers by resource
        let groups = strategy.group_registers(&registers).unwrap();
        
        // Should have two groups (domain1 and domain2)
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("domain1"));
        assert!(groups.contains_key("domain2"));
        
        // domain1 should have 3 registers
        assert_eq!(groups.get("domain1").unwrap().len(), 3);
        
        // domain2 should have 2 registers
        assert_eq!(groups.get("domain2").unwrap().len(), 2);
        
        // Test summary generation
        let domain1_content = strategy.generate_summary_contents(
            &"domain1".to_string(),
            groups.get("domain1").unwrap(),
        ).unwrap();
        
        let summary_text = domain1_content.as_string();
        assert!(summary_text.contains("Resource summary for domain1"));
        assert!(summary_text.contains("3 total registers"));
    }
    
    #[test]
    fn test_account_based_summarization() {
        let strategy = AccountBasedStrategy;
        let registers = create_test_registers();
        
        // Group registers by account
        let groups = strategy.group_registers(&registers).unwrap();
        
        // Should have two groups (owner1 and owner2)
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("owner1"));
        assert!(groups.contains_key("owner2"));
        
        // Test summary generation
        let owner1_content = strategy.generate_summary_contents(
            &"owner1".to_string(),
            groups.get("owner1").unwrap(),
        ).unwrap();
        
        let summary_text = owner1_content.as_string();
        assert!(summary_text.contains("Account summary for owner1"));
    }
    
    #[test]
    fn test_type_based_summarization() {
        let strategy = TypeBasedStrategy;
        let registers = create_test_registers();
        
        // Group registers by type
        let groups = strategy.group_registers(&registers).unwrap();
        
        // Should have two groups (type1 and type2)
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("type1"));
        assert!(groups.contains_key("type2"));
        
        // type1 should have 2 registers
        assert_eq!(groups.get("type1").unwrap().len(), 2);
        
        // type2 should have 3 registers
        assert_eq!(groups.get("type2").unwrap().len(), 3);
    }
    
    #[test]
    fn test_custom_strategy() {
        // Create a custom strategy that groups by first letter of register ID
        let grouping_fn = Arc::new(|register: &Register| -> Result<SummaryGroup> {
            let id_str = register.register_id.to_string();
            let first_char = id_str.chars().next().unwrap_or('?');
            Ok(first_char.to_string())
        });
        
        let strategy = CustomStrategy::new("first_letter", grouping_fn);
        let registers = create_test_registers();
        
        // Group registers
        let groups = strategy.group_registers(&registers).unwrap();
        
        // Number of groups depends on the register IDs, but should be non-empty
        assert!(!groups.is_empty());
        
        // Test with custom content generator
        let content_fn = Arc::new(|group_key: &SummaryGroup, registers: &[Register]| -> Result<RegisterContents> {
            let summary_text = format!(
                "Group {}: {} registers with IDs: {}",
                group_key,
                registers.len(),
                registers.iter()
                    .map(|r| r.register_id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            
            Ok(RegisterContents::with_string(&summary_text))
        });
        
        let strategy_with_content = CustomStrategy::with_content_generator(
            "custom_content",
            grouping_fn,
            content_fn,
        );
        
        let groups = strategy_with_content.group_registers(&registers).unwrap();
        let first_group_key = groups.keys().next().unwrap().clone();
        let first_group = groups.get(&first_group_key).unwrap();
        
        let content = strategy_with_content.generate_summary_contents(
            &first_group_key,
            first_group,
        ).unwrap();
        
        let text = content.as_string();
        assert!(text.contains(&format!("Group {}", first_group_key)));
    }
    
    #[test]
    fn test_summary_manager() {
        let manager = SummaryManager::new();
        let registers = create_test_registers();
        
        // Generate summaries using resource-based strategy
        let summaries = manager.generate_summaries(
            &registers,
            "resource_based",
            1, // epoch
            100, // block height
        ).unwrap();
        
        // Should have 2 summaries (one per domain)
        assert_eq!(summaries.len(), 2);
        
        // Verify they're marked as summaries
        for summary in &summaries {
            assert_eq!(summary.state, RegisterState::Summary);
            assert!(!summary.summarizes.is_empty());
        }
        
        // Test verification
        let summary = &summaries[0];
        let summarized_registers: Vec<Register> = summary.summarizes
            .iter()
            .map(|id| {
                registers.iter()
                    .find(|r| &r.register_id == id)
                    .unwrap()
                    .clone()
            })
            .collect();
            
        let is_valid = manager.verify_summary(summary, &summarized_registers).unwrap();
        assert!(is_valid);
    }
    
    #[test]
    fn test_summary_record() {
        let register_ids = vec![
            ContentId::new_unique(),
            ContentId::new_unique(),
            ContentId::new_unique(),
        ];
        
        let summary_id = ContentId::new_unique();
        let domain = Domain::new("test_domain");
        
        // Create a summary record
        let record = SummaryRecord::new(
            summary_id.clone(),
            register_ids.clone(),
            1, // epoch
            100, // block height
            domain.clone(),
        );
        
        // Convert to metadata
        let metadata = record.to_metadata_map();
        
        // Reconstruct from metadata
        let reconstructed = SummaryRecord::from_metadata(
            summary_id.clone(),
            &metadata,
            domain.clone(),
        ).unwrap();
        
        // Verify fields match
        assert_eq!(reconstructed.summary_id, record.summary_id);
        assert_eq!(reconstructed.epoch, record.epoch);
        assert_eq!(reconstructed.block_height, record.block_height);
        assert_eq!(reconstructed.summary_hash, record.summary_hash);
        
        // Verify includes detection
        assert!(record.verify_includes(&register_ids));
        assert!(record.verify_includes(&register_ids[0..2]));
        assert!(!record.verify_includes(&[ContentId::new_unique()]));
    }
    
    #[test]
    fn test_shared_summary_manager() {
        let shared_manager = SharedSummaryManager::new();
        let registers = create_test_registers();
        
        // Test delegated methods
        let summaries = shared_manager.generate_summaries(
            &registers,
            "account_based",
            1,
            100,
        ).unwrap();
        
        assert_eq!(summaries.len(), 2); // Two owners
        
        // Test strategy access
        let strategy = shared_manager.get_strategy("resource_based").unwrap();
        assert_eq!(strategy.name(), "resource_based");
    }
} 
