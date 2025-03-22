//! This module provides templates for generating TypeScript effect adapter code.
//!
//! These templates are used by the TypeScript code generator to produce
//! a complete TypeScript implementation of an effect adapter, including
//! the main adapter class, type definitions, utility functions, and tests.

use std::collections::HashMap;
use crate::error::Result;
use super::apply_template;

/// Apply a JavaScript template with variables
pub fn apply_js_template(template: &str, variables: &HashMap<String, String>) -> Result<String> {
    apply_template(template, variables)
}

/// Main adapter template for JavaScript
pub const ADAPTER_TEMPLATE: &str = r#"/**
 * {{DOMAIN_PASCALCASE}} Effect Adapter
 * 
 * This adapter provides functionality for interacting with the {{DOMAIN_PASCALCASE}} domain,
 * including applying effects, observing facts, and validating proofs.
 */

import { EffectAdapter, EffectParams, TransactionReceipt, FactType, FactObservationMeta } from 'causality-sdk';
import { {{RPC_STRUCT}}Client } from './rpc_client';

/**
 * {{DOMAIN_PASCALCASE}} adapter implementation
 */
export class {{ADAPTER_NAME}} extends EffectAdapter {
  /**
   * Domain ID for this adapter
   */
  domainId = '{{DOMAIN_ID}}';
  
  /**
   * RPC client instance
   */
  rpcClient;

  /**
   * Configuration for the adapter
   */
  config;

  /**
   * Create a new {{DOMAIN_PASCALCASE}} adapter
   * 
   * @param {string} domainId - Domain identifier
   * @param {Object} config - Adapter configuration
   */
  constructor(domainId = '{{DOMAIN_ID}}', config = {}) {
    super();
    this.domainId = domainId;
    this.config = {
      rpcEndpoint: config.rpcEndpoint || 'https://{{DOMAIN_ENDPOINT}}',
      chainId: config.chainId || '{{CHAIN_ID}}',
      networkId: config.networkId || '{{NETWORK_ID}}',
      gasLimit: config.gasLimit || '{{GAS_LIMIT}}',
      timeout: config.timeout || 30000,
      ...config
    };
    
    this.rpcClient = new {{RPC_STRUCT}}Client(this.config);
  }

  /**
   * Apply an effect to the domain
   * 
   * @param {EffectParams} params - Effect parameters
   * @returns {Promise<TransactionReceipt>} - Transaction receipt
   */
  async applyEffect(params) {
    const effectType = params.effectType;
    
    switch (effectType) {
      {{EFFECT_SWITCH_CASES}}
      default:
        throw new Error(`Unsupported effect type: ${effectType}`);
    }
  }

  /**
   * Observe a fact from the domain
   * 
   * @param {string} factType - Type of fact to observe
   * @param {Object} params - Parameters for fact observation
   * @returns {Promise<{fact: FactType, meta: FactObservationMeta}>} - Observed fact
   */
  async observeFact(factType, params) {
    switch (factType) {
      {{FACT_SWITCH_CASES}}
      default:
        throw new Error(`Unsupported fact type: ${factType}`);
    }
  }

  /**
   * Validate a proof from the domain
   * 
   * @param {string} proofType - Type of proof to validate
   * @param {Uint8Array} proofData - Proof data to validate
   * @returns {Promise<boolean>} - Validation result
   */
  async validateProof(proofType, proofData) {
    switch (proofType) {
      {{PROOF_SWITCH_CASES}}
      default:
        throw new Error(`Unsupported proof type: ${proofType}`);
    }
  }

  {{EFFECT_METHODS}}

  {{FACT_METHODS}}

  {{PROOF_METHODS}}
}
"#;

/// Effect method template for JavaScript
pub const EFFECT_METHOD_TEMPLATE: &str = r#"/**
 * Handle {{EFFECT_TYPE}} effect
 * 
 * @param {Object} params - Effect parameters
 * @returns {Promise<TransactionReceipt>} - Transaction receipt
 */
async handle{{EFFECT_NAME}}(params) {
  // Validate required parameters
  {{REQUIRED_PARAMS_VALIDATION}}
  
  // Prepare transaction data
  const txData = {
    {{TX_DATA_PREPARATION}}
  };
  
  // Send transaction
  const txResponse = await this.rpcClient.sendTransaction(txData);
  
  // Build and return receipt
  return {
    domainId: this.domainId,
    transactionId: txResponse.transactionHash,
    status: txResponse.status ? 'confirmed' : 'failed',
    blockNumber: txResponse.blockNumber,
    timestamp: Date.now(),
    gasUsed: txResponse.gasUsed,
    logs: txResponse.logs,
    metadata: {
      confirmations: 1,
      chainId: this.config.chainId,
      effectType: '{{EFFECT_TYPE}}'
    }
  };
}
"#;

/// Effect switch case template for JavaScript
pub const EFFECT_SWITCH_CASE_TEMPLATE: &str = r#"case '{{EFFECT_TYPE}}':
  return this.handle{{EFFECT_NAME}}(params);
"#;

/// Fact method template for JavaScript
pub const FACT_METHOD_TEMPLATE: &str = r#"/**
 * Observe {{FACT_TYPE}} fact
 * 
 * @param {Object} params - Observation parameters
 * @returns {Promise<{fact: FactType, meta: FactObservationMeta}>} - Observed fact
 */
async observe{{FACT_NAME}}(params) {
  // Validate required parameters
  {{REQUIRED_PARAMS_VALIDATION}}
  
  // Prepare request parameters
  const requestParams = {
    {{REQUEST_PARAMS_PREPARATION}}
  };
  
  // Make RPC request
  const response = await this.rpcClient.{{RPC_METHOD}}(requestParams);
  
  // Extract data
  const data = {{DATA_EXTRACTION}};
  
  // Build and return fact
  return {
    domainId: this.domainId,
    factType: '{{FACT_TYPE}}',
    contentId: `${this.domainId}:{{FACT_TYPE}}:${params.{{IDENTITY_PARAM}}}`,
    data: data,
    timestamp: Date.now(),
    metadata: {
      chainId: this.config.chainId,
      blockNumber: response.blockNumber || 'latest'
    }
  };
}
"#;

/// Fact switch case template for JavaScript
pub const FACT_SWITCH_CASE_TEMPLATE: &str = r#"case '{{FACT_TYPE}}':
  return this.observe{{FACT_NAME}}(params);
"#;

/// Proof method template for JavaScript
pub const PROOF_METHOD_TEMPLATE: &str = r#"/**
 * Validate {{PROOF_TYPE}} proof
 * 
 * @param {Uint8Array} proofData - Proof data
 * @returns {Promise<boolean>} - Validation result
 */
async validate{{PROOF_NAME}}Proof(proofData) {
  try {
    // Parse proof data
    const proof = JSON.parse(new TextDecoder().decode(proofData));
    
    // Validate required fields
    {{REQUIRED_FIELDS_VALIDATION}}
    
    // Verify proof
    {{PROOF_VERIFICATION}}
    
    return true;
  } catch (error) {
    console.error(`Error validating proof: ${error.message}`);
    return false;
  }
}
"#;

/// Proof switch case template for JavaScript
pub const PROOF_SWITCH_CASE_TEMPLATE: &str = r#"case '{{PROOF_TYPE}}':
  return this.validate{{PROOF_NAME}}Proof(proofData);
"#;

/// RPC client template for JavaScript
pub const RPC_CLIENT_TEMPLATE: &str = r#"/**
 * {{DOMAIN_PASCALCASE}} RPC Client
 */
export class {{RPC_STRUCT}}Client {
  /**
   * RPC endpoint URL
   */
  endpoint;
  
  /**
   * Client configuration
   */
  config;
  
  /**
   * Create a new RPC client
   * 
   * @param {Object} config - Client configuration
   */
  constructor(config) {
    this.endpoint = config.rpcEndpoint;
    this.config = config;
  }
  
  /**
   * Make an RPC request
   * 
   * @param {string} method - RPC method name
   * @param {Array} params - RPC method parameters
   * @returns {Promise<any>} - RPC response
   */
  async makeRequest(method, params) {
    const response = await fetch(this.endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(this.config.apiKey ? { 'X-API-Key': this.config.apiKey } : {})
      },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: Date.now(),
        method,
        params
      }),
      timeout: this.config.timeout
    });
    
    const data = await response.json();
    
    if (data.error) {
      throw new Error(`RPC Error: ${data.error.message || JSON.stringify(data.error)}`);
    }
    
    return data.result;
  }
  
  {{RPC_METHODS}}
}
"#;

/// RPC method template for JavaScript
pub const RPC_METHOD_TEMPLATE: &str = r#"/**
 * Call the {{RPC_METHOD}} method
 * 
 * @param {Object} params - Method parameters
 * @returns {Promise<any>} - Method result
 */
async {{RPC_METHOD_NAME}}(params) {
  return this.makeRequest('{{RPC_METHOD}}', [params]);
}
"#;

/// Types template for JavaScript
pub const TYPES_TEMPLATE: &str = r#"/**
 * Type definitions for {{DOMAIN_PASCALCASE}} adapter
 */

/**
 * {{DOMAIN_PASCALCASE}} transaction
 * 
 * @typedef {Object} Transaction
 * @property {string} from - Sender address
 * @property {string} to - Recipient address
 * @property {string} value - Transaction value
 * @property {string} data - Transaction data
 * @property {string} gas - Gas limit
 * @property {string} gasPrice - Gas price
 * @property {string} nonce - Transaction nonce
 */

/**
 * {{DOMAIN_PASCALCASE}} transaction receipt
 * 
 * @typedef {Object} TransactionReceipt
 * @property {string} transactionHash - Transaction hash
 * @property {number} blockNumber - Block number
 * @property {string} status - Transaction status
 * @property {string} gasUsed - Gas used
 * @property {Array<Object>} logs - Event logs
 */

/**
 * {{DOMAIN_PASCALCASE}} block
 * 
 * @typedef {Object} Block
 * @property {string} hash - Block hash
 * @property {number} number - Block number
 * @property {string} timestamp - Block timestamp
 * @property {Array<string>} transactions - Transaction hashes
 */

/**
 * {{DOMAIN_PASCALCASE}} proof
 * 
 * @typedef {Object} Proof
 * @property {string} type - Proof type
 * @property {Object} data - Proof data
 * @property {string} signature - Cryptographic signature
 */

// Export types
export {
  Transaction,
  TransactionReceipt,
  Block,
  Proof
};
"#;

/// Utils template for JavaScript
pub const UTILS_TEMPLATE: &str = r#"/**
 * Utility functions for {{DOMAIN_PASCALCASE}} adapter
 */

/**
 * Convert hex string to decimal string
 * 
 * @param {string} hexStr - Hex string to convert
 * @returns {string} - Decimal string
 */
export function hexToDecimal(hexStr) {
  if (!hexStr) return '0';
  const hex = hexStr.startsWith('0x') ? hexStr.slice(2) : hexStr;
  return BigInt(`0x${hex}`).toString(10);
}

/**
 * Convert decimal string to hex string
 * 
 * @param {string} decStr - Decimal string to convert
 * @returns {string} - Hex string
 */
export function decimalToHex(decStr) {
  if (!decStr) return '0x0';
  const hex = BigInt(decStr).toString(16);
  return `0x${hex}`;
}

/**
 * Parse JSON-RPC response
 * 
 * @param {Object} response - JSON-RPC response
 * @returns {Object} - Parsed result
 */
export function parseRpcResponse(response) {
  if (response.error) {
    throw new Error(`RPC Error: ${response.error.message || JSON.stringify(response.error)}`);
  }
  return response.result;
}

/**
 * Build authentication headers
 * 
 * @param {Object} config - Adapter configuration
 * @returns {Object} - Headers object
 */
export function buildAuthHeaders(config) {
  const headers = {
    'Content-Type': 'application/json',
  };
  
  if (config.apiKey) {
    headers['X-API-Key'] = config.apiKey;
  }
  
  return headers;
}

/**
 * Format parameters for RPC request
 * 
 * @param {Object} params - Parameters to format
 * @returns {Object} - Formatted parameters
 */
export function formatRpcParams(params) {
  // Basic formatting for common parameters
  const result = { ...params };
  
  // Convert numeric strings to hex if needed
  for (const key of ['value', 'gasLimit', 'gasPrice']) {
    if (result[key] && !result[key].startsWith('0x')) {
      result[key] = decimalToHex(result[key]);
    }
  }
  
  return result;
}
"#;

/// Adapter test template for JavaScript
pub const ADAPTER_TEST_TEMPLATE: &str = r#"/**
 * Tests for {{DOMAIN_PASCALCASE}} adapter
 */

import { {{ADAPTER_NAME}} } from '../{{DOMAIN_ID}}_adapter';

describe('{{ADAPTER_NAME}}', () => {
  let adapter;
  
  beforeEach(() => {
    adapter = new {{ADAPTER_NAME}}();
    // Mock RPC calls
    adapter.rpcClient.makeRequest = jest.fn().mockImplementation((method, params) => {
      // Return mock data based on method
      switch (method) {
        case 'eth_getBalance':
          return Promise.resolve('0x1000');
        case 'eth_sendTransaction':
          return Promise.resolve({
            transactionHash: '0x123456',
            status: '0x1',
            blockNumber: 100,
            gasUsed: '0x5000'
          });
        default:
          return Promise.resolve({});
      }
    });
  });
  
  test('should initialize with default configuration', () => {
    expect(adapter.domainId).toBe('{{DOMAIN_ID}}');
    expect(adapter.config.rpcEndpoint).toBeDefined();
    expect(adapter.config.chainId).toBeDefined();
  });
  
  test('should initialize with custom configuration', () => {
    const customConfig = {
      rpcEndpoint: 'https://custom-rpc.example.com',
      chainId: '999',
      networkId: '999',
      gasLimit: '1000000'
    };
    
    const customAdapter = new {{ADAPTER_NAME}}('custom-domain', customConfig);
    expect(customAdapter.domainId).toBe('custom-domain');
    expect(customAdapter.config.rpcEndpoint).toBe('https://custom-rpc.example.com');
    expect(customAdapter.config.chainId).toBe('999');
  });
  
  {{ADAPTER_TEST_METHODS}}
});
"#;

/// Effect test template for JavaScript
pub const EFFECT_TEST_TEMPLATE: &str = r#"/**
 * Tests for {{DOMAIN_PASCALCASE}} adapter effects
 */

import { {{ADAPTER_NAME}} } from '../{{DOMAIN_ID}}_adapter';

describe('{{ADAPTER_NAME}} Effects', () => {
  let adapter;
  
  beforeEach(() => {
    adapter = new {{ADAPTER_NAME}}();
    // Mock RPC calls
    adapter.rpcClient.makeRequest = jest.fn().mockImplementation((method, params) => {
      // Return mock data based on method
      switch (method) {
        case 'eth_sendTransaction':
          return Promise.resolve({
            transactionHash: '0x123456',
            status: '0x1',
            blockNumber: 100,
            gasUsed: '0x5000',
            logs: []
          });
        default:
          return Promise.resolve({});
      }
    });
  });
  
  {{EFFECT_TEST_METHODS}}
});
"#;

/// Fact test template for JavaScript
pub const FACT_TEST_TEMPLATE: &str = r#"/**
 * Tests for {{DOMAIN_PASCALCASE}} adapter facts
 */

import { {{ADAPTER_NAME}} } from '../{{DOMAIN_ID}}_adapter';

describe('{{ADAPTER_NAME}} Facts', () => {
  let adapter;
  
  beforeEach(() => {
    adapter = new {{ADAPTER_NAME}}();
    // Mock RPC calls
    adapter.rpcClient.makeRequest = jest.fn().mockImplementation((method, params) => {
      // Return mock data based on method
      switch (method) {
        case 'eth_getBalance':
          return Promise.resolve('0x1000');
        case 'eth_getTransactionByHash':
          return Promise.resolve({
            hash: '0x123456',
            from: '0xabcdef',
            to: '0xfedcba',
            value: '0x1000',
            blockNumber: '0x100'
          });
        default:
          return Promise.resolve({});
      }
    });
  });
  
  {{FACT_TEST_METHODS}}
});
"#;

/// Proof test template for JavaScript
pub const PROOF_TEST_TEMPLATE: &str = r#"/**
 * Tests for {{DOMAIN_PASCALCASE}} adapter proofs
 */

import { {{ADAPTER_NAME}} } from '../{{DOMAIN_ID}}_adapter';

describe('{{ADAPTER_NAME}} Proofs', () => {
  let adapter;
  
  beforeEach(() => {
    adapter = new {{ADAPTER_NAME}}();
    // Mock RPC calls
    adapter.rpcClient.makeRequest = jest.fn().mockImplementation((method, params) => {
      // Return mock data based on method
      return Promise.resolve({});
    });
  });
  
  {{PROOF_TEST_METHODS}}
});
"#;

/// README template for JavaScript
pub const README_TEMPLATE: &str = r#"# {{ADAPTER_NAME}}

This adapter provides functionality for interacting with the {{DOMAIN_PASCALCASE}} domain,
including applying effects, observing facts, and validating proofs.

## Features

- Apply effects like transfers, contract calls, and deployments
- Observe facts like balances, transactions, blocks, and contract state
- Validate proofs of transactions, receipts, and accounts

## Installation

```bash
npm install causality-{{DOMAIN_ID}}-adapter
```

## Usage

```javascript
import { {{ADAPTER_NAME}} } from 'causality-{{DOMAIN_ID}}-adapter';

// Create adapter with custom configuration
const adapter = new {{ADAPTER_NAME}}('{{DOMAIN_ID}}', {
  rpcEndpoint: 'https://{{DOMAIN_ENDPOINT}}',
  chainId: '{{CHAIN_ID}}',
  networkId: '{{NETWORK_ID}}',
  gasLimit: '2000000',
  apiKey: 'your-api-key'
});

// Apply an effect
const receipt = await adapter.applyEffect({
  effectType: 'transfer',
  params: {
    from: '0x1234...',
    to: '0x5678...',
    value: '1000000000000000000' // 1 ETH in wei
  },
  metadata: {}
});

// Observe a fact
const fact = await adapter.observeFact('balance', {
  address: '0x1234...'
});

// Validate a proof
const isValid = await adapter.validateProof('transaction', proofData);
```

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| rpcEndpoint | RPC endpoint URL | https://{{DOMAIN_ENDPOINT}} |
| chainId | Chain ID | {{CHAIN_ID}} |
| networkId | Network ID | {{NETWORK_ID}} |
| gasLimit | Default gas limit | {{GAS_LIMIT}} |
| timeout | Request timeout in ms | 30000 |
| apiKey | API key for authentication | null |

## API Reference

See the [API documentation](./docs/API.md) for detailed information on all available methods.

## License

MIT
"#;

/// API documentation template for JavaScript
pub const API_DOCS_TEMPLATE: &str = r#"# {{ADAPTER_NAME}} API Reference

## Configuration

The adapter can be configured with the following options:

```javascript
const adapter = new {{ADAPTER_NAME}}('{{DOMAIN_ID}}', {
  rpcEndpoint: 'https://{{DOMAIN_ENDPOINT}}',
  chainId: '{{CHAIN_ID}}',
  networkId: '{{NETWORK_ID}}',
  gasLimit: '2000000',
  apiKey: 'your-api-key',
  timeout: 30000
});
```

## Effects

The adapter supports the following effects:

{{EFFECTS_DOCUMENTATION}}

## Facts

The adapter supports the following facts:

{{FACTS_DOCUMENTATION}}

## Proofs

The adapter supports the following proofs:

{{PROOFS_DOCUMENTATION}}

## Error Handling

All methods return Promises that may be rejected with an error. Common error scenarios:

- RPC endpoint unreachable
- Invalid parameters
- Authentication failure
- Network congestion
- Contract execution errors

Example error handling:

```javascript
try {
  const receipt = await adapter.applyEffect({
    effectType: 'transfer',
    params: { /* ... */ }
  });
} catch (error) {
  console.error(`Effect failed: ${error.message}`);
  // Handle the error appropriately
}
```
"#;

/// Basic example template for JavaScript
pub const BASIC_EXAMPLE_TEMPLATE: &str = r#"/**
 * Basic example of using the {{ADAPTER_NAME}}
 */

import { {{ADAPTER_NAME}} } from 'causality-{{DOMAIN_ID}}-adapter';

async function main() {
  // Create adapter with custom configuration
  const adapter = new {{ADAPTER_NAME}}('{{DOMAIN_ID}}', {
    rpcEndpoint: 'https://{{DOMAIN_ENDPOINT}}',
    chainId: '{{CHAIN_ID}}',
    networkId: '{{NETWORK_ID}}',
    gasLimit: '2000000',
    apiKey: process.env.API_KEY
  });
  
  try {
    // Example: Check balance
    console.log('Checking balance...');
    const balanceFact = await adapter.observeFact('balance', {
      address: '0x1234567890123456789012345678901234567890'
    });
    console.log('Balance:', balanceFact.data);
    
    // Example: Transfer tokens
    console.log('Transferring tokens...');
    const receipt = await adapter.applyEffect({
      effectType: 'transfer',
      params: {
        from: '0x1234567890123456789012345678901234567890',
        to: '0x0987654321098765432109876543210987654321',
        value: '1000000000000000000' // 1 token
      }
    });
    console.log('Transfer complete:', receipt.transactionId);
    
    // Example: Validate transaction proof
    console.log('Validating proof...');
    const proof = Buffer.from(JSON.stringify({
      type: 'transaction',
      transactionHash: receipt.transactionId,
      blockHash: receipt.blockHash,
      signature: '0x...'
    }));
    const isValid = await adapter.validateProof('transaction', proof);
    console.log('Proof valid:', isValid);
    
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main().catch(console.error);
"#;

/// NPM package template for JavaScript
pub const PACKAGE_JSON_TEMPLATE: &str = r#"{
  "name": "causality-{{DOMAIN_ID}}-adapter",
  "version": "0.1.0",
  "description": "{{DOMAIN_PASCALCASE}} adapter for Causality",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "test": "jest",
    "lint": "eslint src --ext .ts,.js",
    "prepublish": "npm run build"
  },
  "keywords": [
    "causality",
    "{{DOMAIN_ID}}",
    "adapter",
    "blockchain"
  ],
  "author": "Causality Team",
  "license": "MIT",
  "dependencies": {
    "causality-sdk": "^0.1.0",
    "node-fetch": "^2.6.7"
  },
  "devDependencies": {
    "@types/jest": "^27.4.0",
    "@types/node": "^17.0.10",
    "eslint": "^8.7.0",
    "jest": "^27.4.7",
    "ts-jest": "^27.1.3",
    "typescript": "^4.5.5"
  }
}
"#;

/// TypeScript definition template for JavaScript
pub const TYPESCRIPT_DEFINITION_TEMPLATE: &str = r#"/**
 * TypeScript definitions for {{DOMAIN_PASCALCASE}} adapter
 */

import { EffectAdapter, EffectParams, TransactionReceipt, FactType, FactObservationMeta } from 'causality-sdk';

/**
 * Configuration for {{DOMAIN_PASCALCASE}} adapter
 */
export interface {{DOMAIN_PASCALCASE}}Config {
  /**
   * RPC endpoint URL
   */
  rpcEndpoint?: string;
  
  /**
   * Chain ID
   */
  chainId?: string;
  
  /**
   * Network ID
   */
  networkId?: string;
  
  /**
   * Default gas limit
   */
  gasLimit?: string;
  
  /**
   * Request timeout in ms
   */
  timeout?: number;
  
  /**
   * API key for authentication
   */
  apiKey?: string;
  
  /**
   * Additional parameters
   */
  [key: string]: any;
}

/**
 * {{DOMAIN_PASCALCASE}} adapter implementation
 */
export class {{ADAPTER_NAME}} extends EffectAdapter {
  /**
   * Domain ID for this adapter
   */
  domainId: string;
  
  /**
   * Configuration for the adapter
   */
  config: {{DOMAIN_PASCALCASE}}Config;
  
  /**
   * RPC client instance
   */
  rpcClient: any;
  
  /**
   * Create a new {{DOMAIN_PASCALCASE}} adapter
   */
  constructor(domainId?: string, config?: {{DOMAIN_PASCALCASE}}Config);
  
  /**
   * Apply an effect to the domain
   */
  applyEffect(params: EffectParams): Promise<TransactionReceipt>;
  
  /**
   * Observe a fact from the domain
   */
  observeFact(factType: string, params: Record<string, any>): Promise<{fact: FactType, meta: FactObservationMeta}>;
  
  /**
   * Validate a proof from the domain
   */
  validateProof(proofType: string, proofData: Uint8Array): Promise<boolean>;
  
  {{TS_EFFECT_METHODS}}
  
  {{TS_FACT_METHODS}}
  
  {{TS_PROOF_METHODS}}
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apply_js_template() {
        let template = "Hello, {{NAME}}!";
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "World".to_string());
        
        let result = apply_js_template(template, &vars).unwrap();
        assert_eq!(result, "Hello, World!");
    }
    
    #[test]
    fn test_template_constants_exist() {
        assert!(!ADAPTER_TEMPLATE.is_empty());
        assert!(!EFFECT_METHOD_TEMPLATE.is_empty());
        assert!(!FACT_METHOD_TEMPLATE.is_empty());
        assert!(!PROOF_METHOD_TEMPLATE.is_empty());
        assert!(!RPC_CLIENT_TEMPLATE.is_empty());
        assert!(!TYPES_TEMPLATE.is_empty());
        assert!(!UTILS_TEMPLATE.is_empty());
        assert!(!ADAPTER_TEST_TEMPLATE.is_empty());
        assert!(!EFFECT_TEST_TEMPLATE.is_empty());
        assert!(!FACT_TEST_TEMPLATE.is_empty());
        assert!(!PROOF_TEST_TEMPLATE.is_empty());
        assert!(!README_TEMPLATE.is_empty());
        assert!(!API_DOCS_TEMPLATE.is_empty());
        assert!(!BASIC_EXAMPLE_TEMPLATE.is_empty());
        assert!(!PACKAGE_JSON_TEMPLATE.is_empty());
        assert!(!TYPESCRIPT_DEFINITION_TEMPLATE.is_empty());
    }
} 