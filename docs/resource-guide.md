# Resource and Domain Model Guide

This guide provides a detailed overview of resources and domain modeling in the Temporal Effect Language (TEL).

## Resources in TEL

Resources are the fundamental building blocks in TEL. They represent entities in your domain as content-addressed, immutable data structures.

### Resource Schema

A resource schema defines the structure of a resource type, including field definitions, types, and requirements.

```
let userSchema = ResourceSchema.new("User")
  .addField("id", FieldType.String, true)
  .addField("name", FieldType.String, true)
  .addField("email", FieldType.String, true)
  .addField("age", FieldType.Integer, false);
```

In this example:
- `"User"` is the resource type name
- Each field has a name, type, and a boolean indicating whether it's required
- Types include String, Integer, Float, Boolean, Array, and Object

### Resource Creation

Resources are created as simple objects that conform to their schema:

```
let user = {
  "id": "user-1",
  "name": "John Doe",
  "email": "john@example.com",
  "age": 30
};
```

### Content Addressing

In TEL, resources are content-addressed, meaning they are identified by a hash of their content. When you store a resource, you receive a content ID:

```
let userId = Store(user);
```

This content ID can be used to retrieve the resource later:

```
let loadedUser = Load(userId);
```

The content ID ensures that:
1. Resources are immutable - any change results in a new content ID
2. Resources can be verified for integrity
3. Identical resources are naturally deduplicated

## Domain Modeling

A domain model in TEL represents a collection of resource types and their relationships.

### Creating a Domain Model

```
let ecommerceModel = DomainModel.new("Ecommerce")
  .addResourceType(userSchema)
  .addResourceType(productSchema)
  .addResourceType(orderSchema)
  .addResourceType(orderItemSchema);
```

### Defining Relationships

Relationships define how resource types are connected in the domain:

```
// One-to-many relationship (one user can have many orders)
ecommerceModel.addRelationship("UserOrders", "User", "Order", "1:N");

// Many-to-many relationship (products can be in many categories, and categories can contain many products)
ecommerceModel.addRelationship("ProductCategories", "Product", "Category", "N:N");
```

Relationship types include:
- "1:1": One-to-one
- "1:N": One-to-many
- "N:1": Many-to-one
- "N:N": Many-to-many

### Validation Rules

Validation rules ensure that resources and their relationships conform to domain-specific requirements.

#### Field Validation

```
// Ensure email contains '@'
ecommerceModel.addValidationRule(FieldValidationRule.new("ValidEmailRule")
  .forResourceType("User")
  .onField("email")
  .withValidation(value => value.includes("@")));

// Ensure price is positive
ecommerceModel.addValidationRule(FieldValidationRule.new("PositivePriceRule")
  .forResourceType("Product")
  .onField("price")
  .withValidation(value => value > 0));
```

#### Relationship Validation

```
// Ensure orders have at least one order item
ecommerceModel.addValidationRule(RelationshipValidationRule.new("OrderItemsRule")
  .forRelationship("OrderItems")
  .withValidation((order, items) => items.length > 0));
```

### Constraints

Constraints are enforced rules that must be satisfied by the domain model.

#### Unique Field Constraint

```
// Ensure email addresses are unique across users
ecommerceModel.addConstraint(UniqueFieldConstraint.new("UniqueEmailConstraint")
  .forResourceType("User")
  .onField("email"));
```

#### Relationship Cardinality Constraint

```
// Ensure orders have at least one item and no maximum limit
ecommerceModel.addConstraint(RelationshipCardinalityConstraint.new("OrderItemsCardinality")
  .forRelationship("OrderItems")
  .withMinCardinality(1)
  .withMaxCardinality(null));
```

## Working with Resources and Domain Models

### Storing and Retrieving Resources

```
// Store a resource
let productId = Store(product);

// Load a resource
let loadedProduct = Load(productId);
```

### Querying Resources

```
// Find all active products
let activeProducts = Query()
  .fromResourceType("Product")
  .where("active", "=", true)
  .execute();

// Find recent orders for a user
let userOrders = Query()
  .fromResourceType("Order")
  .where("user_id", "=", userId)
  .and("created_at", ">", "2023-01-01")
  .orderBy("created_at", "desc")
  .execute();
```

### Validating Resources Against a Domain Model

```
// Validate a resource against schema and rules
let validationResult = ecommerceModel.validateResource(product);

// Validate the entire domain model
let domainValidation = ecommerceModel.validate();
```

### Checking Constraints

```
// Check if a resource satisfies constraints
let constraintResults = ecommerceModel.checkConstraints(user);

// Check constraints across the entire domain
let domainConstraints = ecommerceModel.checkAllConstraints();
```

## Best Practices

### Resource Design

1. **Unique Identifiers**: Always include an ID field that uniquely identifies the resource.
2. **Required Fields**: Mark fields as required only if they are truly necessary.
3. **Immutability**: Design resources to be immutable, with changes creating new versions.
4. **Natural Keys**: Use natural keys when possible to make resources more identifiable.

### Relationship Design

1. **Meaningful Names**: Give relationships descriptive names that convey their purpose.
2. **Cardinality**: Choose the appropriate cardinality for relationships.
3. **Foreign Keys**: Use consistent field names for foreign keys (e.g., `user_id` for a relationship to a User).

### Validation and Constraints

1. **Validation Early**: Apply validation as early as possible in the resource lifecycle.
2. **Meaningful Messages**: Provide helpful error messages in validation rules.
3. **Layered Validation**: Use a combination of field-level and relationship-level validations.

## Example: E-commerce Domain Model

Here's a complete example of an e-commerce domain model:

```
// Define resource schemas
let userSchema = ResourceSchema.new("User")
  .addField("id", FieldType.String, true)
  .addField("name", FieldType.String, true)
  .addField("email", FieldType.String, true);

let productSchema = ResourceSchema.new("Product")
  .addField("id", FieldType.String, true)
  .addField("name", FieldType.String, true)
  .addField("price", FieldType.Float, true)
  .addField("stock", FieldType.Integer, true);

let orderSchema = ResourceSchema.new("Order")
  .addField("id", FieldType.String, true)
  .addField("user_id", FieldType.String, true)
  .addField("total", FieldType.Float, true)
  .addField("status", FieldType.String, true);

let orderItemSchema = ResourceSchema.new("OrderItem")
  .addField("id", FieldType.String, true)
  .addField("order_id", FieldType.String, true)
  .addField("product_id", FieldType.String, true)
  .addField("quantity", FieldType.Integer, true)
  .addField("price", FieldType.Float, true);

// Create domain model
let ecommerceModel = DomainModel.new("Ecommerce")
  .addResourceType(userSchema)
  .addResourceType(productSchema)
  .addResourceType(orderSchema)
  .addResourceType(orderItemSchema);

// Define relationships
ecommerceModel.addRelationship("UserOrders", "User", "Order", "1:N");
ecommerceModel.addRelationship("OrderItems", "Order", "OrderItem", "1:N");
ecommerceModel.addRelationship("ProductItems", "Product", "OrderItem", "1:N");

// Add validation rules
ecommerceModel.addValidationRule(FieldValidationRule.new("ValidEmailRule")
  .forResourceType("User")
  .onField("email")
  .withValidation(value => value.includes("@")));

ecommerceModel.addValidationRule(FieldValidationRule.new("PositivePriceRule")
  .forResourceType("Product")
  .onField("price")
  .withValidation(value => value > 0));

ecommerceModel.addValidationRule(FieldValidationRule.new("ValidStatusRule")
  .forResourceType("Order")
  .onField("status")
  .withValidation(value => ["pending", "shipped", "delivered", "cancelled"].includes(value)));

// Add constraints
ecommerceModel.addConstraint(UniqueFieldConstraint.new("UniqueEmailConstraint")
  .forResourceType("User")
  .onField("email"));

ecommerceModel.addConstraint(RelationshipCardinalityConstraint.new("OrderItemsCardinality")
  .forRelationship("OrderItems")
  .withMinCardinality(1)
  .withMaxCardinality(null));
```

This domain model provides a solid foundation for an e-commerce application, with clearly defined resources, relationships, validation rules, and constraints. 