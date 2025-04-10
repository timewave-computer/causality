# TEL-TEG Adjunction

This document illustrates the categorical adjunction between the Temporal Effect Language (TEL) and the Temporal Effect Graph (TEG) intermediate representation.

## Functors F and G

The TEL-TEG adjunction involves two functors:
- `F: TEL → TEG` translates TEL programs to TEG
- `G: TEG → TEL` translates TEG back to TEL programs

```mermaid
graph LR
    subgraph TEL
        A[TEL Program]
        C[TEL Program']
    end
    
    subgraph TEG
        B[Temporal Effect Graph]
        D[Temporal Effect Graph']
    end
    
    A -- "F" --> B
    B -- "G" --> C
    
    A -- "f" --> C
    B -- "g" --> D
    
    C -- "F" --> D
```

## Natural Isomorphism

The key property of the adjunction is the natural isomorphism between the morphism sets:

```mermaid
graph TB
    subgraph "Hom_TEL(A, G(B))"
        f1[TEL morphism]
        f2[TEL morphism]
        f3[...]
    end
    
    subgraph "Hom_TEG(F(A), B)"
        g1[TEG morphism]
        g2[TEG morphism]
        g3[...]
    end
    
    f1 -- "φ (isomorphism)" --> g1
    f2 -- "φ (isomorphism)" --> g2
    f3 -- "φ (isomorphism)" --> g3
    
    g1 -. "φ⁻¹" .-> f1
    g2 -. "φ⁻¹" .-> f2
    g3 -. "φ⁻¹" .-> f3
```

## Adjunction Unit and Counit

The adjunction is characterized by its unit and counit natural transformations:

```mermaid
graph LR
    subgraph "Unit: η: 1_TEL ⟹ G ∘ F"
        A1[TEL Program]
        B1[G(F(TEL Program))]
        A1 -- "η_A" --> B1
    end
    
    subgraph "Counit: ε: F ∘ G ⟹ 1_TEG"
        A2[F(G(TEG Program))]
        B2[TEG Program]
        A2 -- "ε_B" --> B2
    end
```

## TEG Structure

The internal structure of a TEG includes effect nodes, resource nodes, and the edges connecting them:

```mermaid
graph TD
    subgraph "Temporal Effect Graph"
        E1[Effect Node 1] -- "continuation" --> E2[Effect Node 2]
        E1 -- "uses" --> R1[Resource Node 1]
        E2 -- "modifies" --> R2[Resource Node 2]
        R1 -- "depends on" --> R2
        E3[Effect Node 3] -- "continuation" --> E1
    end
```

## Content Addressing

The content addressing property ensures semantic equivalence:

```mermaid
graph LR
    subgraph "Content Addressing"
        A[TEL Program]
        B[TEG Program]
        C[TEL Program']
        
        A -- "F" --> B
        B -- "G" --> C
        
        A -. "semantically\nequivalent" .-> C
        
        H1[Content Hash 1] --- A
        H2[Content Hash 2] --- B
        H1 -. "derives" .-> H2
    end
```

## Triangle Identities

The triangle identities validate the adjunction:

```mermaid
graph LR
    subgraph "Triangle Identity 1"
        F -- "F ⋅ η" --> FGF
        F -- "ε_F" --> F
    end
    
    subgraph "Triangle Identity 2"
        G -- "η_G" --> GFG
        G -- "G ⋅ ε" --> G
    end
    
    F[Functor F]
    G[Functor G]
    FGF[F ∘ G ∘ F]
    GFG[G ∘ F ∘ G]
``` 