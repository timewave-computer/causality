# Temporal Effect Language (TEL) Reference

## Introduction

The Temporal Effect Language (TEL) is a domain-specific language designed for modeling resources, relationships, and temporal effects in content-addressed systems. This reference provides a comprehensive overview of TEL's syntax, semantics, and features.

## Basic Syntax

### Literals

```
// String literals
"Hello, world!"
'Single quotes work too'

// Number literals
42          // Integer
3.14        // Float
1.5e6       // Scientific notation

// Boolean literals
true
false

// Null
null

// Array literals
[1, 2, 3]
["apple", "banana", "cherry"]
[true, false, 42, "mixed types"]

// Object literals
{
  "name": "John",
  "age": 30,
  "isAdmin": true
}
```

### Variables and Let Bindings

```
// Simple binding
let x = 42;

// Multiple bindings
let name = "John";
let age = 30;
let isAdmin = true;

// Nested bindings
let user = {
  "id": "user-1",
  "profile": {
    "name": "John Doe",
    "email": "john@example.com"
  }
};

// Binding with expressions
let a = 5;
let b = 10;
let sum = a + b;
```

### Functions and Lambdas

```
// Simple function
let add = (a, b) => a + b;

// Multi-line function
let calculate = (a, b) => {
  let sum = a + b;
  let product = a * b;
  return {
    "sum": sum,
    "product": product
  };
};

// Higher-order function
let applyTwice = (f, x) => f(f(x));

// Recursive function
let factorial = (n) => {
  if (n <= 1) {
    return 1;
  } else {
    return n * factorial(n-1);
  }
};
```

### Control Flow

```
// Conditional expression
let status = if (age >= 18) "adult" else "minor";

// If statement
if (temperature > 30) {
  "It's hot outside";
} else if (temperature < 0) {
  "It's freezing";
} else {
  "The temperature is moderate";
}

// Match expression
match (day) {
  "Monday" => "Start of work week",
  "Friday" => "End of work week",
  "Saturday" | "Sunday" => "Weekend",
  _ => "Regular weekday"
}
```

## Resource Operations

### Resource Schema Definition

```
let userSchema = ResourceSchema.new("User")
  .addField("id", FieldType.String, true)
  .addField("name", FieldType.String, true)
  .addField("email", FieldType.String, true)
  .addField("age", FieldType.Integer, false);
```

### Resource Creation and Storage

```
let user = {
  "id": "user-1",
  "name": "John Doe",
  "email": "john@example.com",
  "age": 30
};

// Store the resource and get a content ID
let userId = Store(user);
```

### Resource Retrieval

```
// Load a resource by its content ID
let loadedUser = Load(userId);

// Query resources
let activeUsers = Query()
  .fromResourceType("User")
  .where("active", "=", true)
  .orderBy("name", "asc")
  .limit(10)
  .execute();
```

## Domain Modeling

### Domain Model Definition

```
let ecommerceModel = DomainModel.new("Ecommerce")
  .addResourceType(userSchema)
  .addResourceType(productSchema)
  .addResourceType(orderSchema);
```

### Relationship Definition

```
// Define a one-to-many relationship
ecommerceModel.addRelationship("UserOrders", "User", "Order", "1:N");

// Define a many-to-many relationship
ecommerceModel.addRelationship("ProductCategories", "Product", "Category", "N:N");
```

### Validation Rules

```
// Field validation rule
ecommerceModel.addValidationRule(FieldValidationRule.new("ValidEmailRule")
  .forResourceType("User")
  .onField("email")
  .withValidation(value => value.includes("@")));

// Relationship validation rule
ecommerceModel.addValidationRule(RelationshipValidationRule.new("OrderItemsRule")
  .forRelationship("OrderItems")
  .withValidation((order, items) => items.length > 0));
```

### Constraints

```
// Unique field constraint
ecommerceModel.addConstraint(UniqueFieldConstraint.new("UniqueEmailConstraint")
  .forResourceType("User")
  .onField("email"));

// Relationship cardinality constraint
ecommerceModel.addConstraint(RelationshipCardinalityConstraint.new("OrderItemsCardinality")
  .forRelationship("OrderItems")
  .withMinCardinality(1)
  .withMaxCardinality(null));
```

## Integration with Host Languages

### Host Function Calls

```
// Call a function in the host environment
let currentTime = Host.call("getCurrentTime");
let randomNumber = Host.call("random", 1, 100);
```

### Function Registration

```
// Register a TEL function for the host to call
let add = (a, b) => a + b;
Host.register("addNumbers", add);
```

### Working with Host Data

```
// Process data from the host
let processHostData = (data) => {
  let processed = data.map(item => item * 2);
  Host.call("log", "Processed data");
  return processed;
};
```

## Querying

### Basic Query

```
let users = Query()
  .fromResourceType("User")
  .execute();
```

### Filtered Query

```
let activeUsers = Query()
  .fromResourceType("User")
  .where("active", "=", true)
  .execute();
```

### Complex Query

```
let recentOrders = Query()
  .fromResourceType("Order")
  .where("status", "=", "pending")
  .and("created_at", ">", "2023-01-01")
  .orderBy("created_at", "desc")
  .limit(10)
  .execute();
```

### Joins

```
let userOrders = Query()
  .fromResourceType("User")
  .join("Order", "id", "user_id")
  .where("Order.status", "=", "pending")
  .execute();
```

## Error Handling

```
// Try-catch pattern
try {
  let result = dangerousOperation();
  return result;
} catch (error) {
  Host.call("log", "Error occurred: " + error);
  return null;
}
```

## Best Practices

1. **Content Addressing**: Always design resources to be content-addressed.
   
2. **Immutability**: Treat resources as immutable. Create new versions instead of modifying existing ones.
   
3. **Schema First**: Define resource schemas before creating resources to ensure consistency.
   
4. **Validation**: Use validation rules to maintain data integrity.
   
5. **Relationships**: Model relationships explicitly to represent domain connections.
   
6. **Queries**: Write specific queries that return only the data you need.
   
7. **Host Integration**: Keep the boundary between TEL and the host language clean.

## Further Reading

- [Resource and Domain Model Guide](resource-guide.md)
- [Runtime Integration Guide](runtime-guide.md)
- [Examples](../examples/README.md) 