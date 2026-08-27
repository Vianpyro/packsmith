# Port types and sequencing

- **Spec version:** 0 (unstable; Phase 0)
- **Status:** normative
- **Decided by:** ADR-0016. Constrained by ADR-0003, ADR-0007, ADR-0010, ADR-0012, ADR-0013.
- **Licence:** `MIT OR Apache-2.0`, as all of `spec/`.

This document defines what a port may hold, what may connect to what, and how a graph
expresses the order of statements. It is the contract every SDK and every host implements.
It defines no file formats; the JSON Schemas are derived from it, not the reverse.

Where this document says MUST, a conforming implementation rejects the input with a
diagnostic. Where it says MAY, the behaviour is the block author's to decide.

---

## 1. Structure of a graph

A **node** is one instance of a block. Every node has a globally unique id within its project.

A node is exactly one of:

- a **statement node**, which does something in order and produces no value;
- a **value node**, which produces one or more values and does nothing in order.

The block manifest decides which. A node MUST NOT be both.

A node has:

- **input ports**, each typed, each holding either a *literal* or a *data edge*, never both
  and never neither unless the port is optional;
- **output ports**, each typed, present only on value nodes;
- **slots**, named ordered collections of child statement nodes, present only on nodes that
  declare them.

Two mechanisms, two meanings, no overlap:

| Mechanism | Means | Shape |
|---|---|---|
| Slot | order, containment, "then" | array of child nodes |
| Data edge | a value flowing from an output to an input | reference by node id and port name |

**Data edges carry no ordering meaning.** Blocks are pure (ADR-0005), so evaluating a value
node has no observable effect and its position in time is not a thing a user can perceive.
Anything a user can observe happening in a sequence happens because a statement node sits at
an index in a slot.

---

## 2. Sequencing

### 2.1 Slots

A slot is declared in a block manifest with a name and the type `body` (section 4.11). Its
value in a graph is a JSON array of child statement nodes, in execution order.

```json
{
  "id": "n_tick",
  "block": "packsmith/function@1.0.0",
  "inputs": { "name": "example:tick" },
  "slots": {
    "body": [
      { "id": "n_1", "block": "packsmith/raw-command@1.0.0",
        "inputs": { "command": "say hello" } },
      { "id": "n_2", "block": "packsmith/as-each@1.0.0",
        "inputs": { "targets": "@e[type=minecraft:zombie]" },
        "slots": { "run": [
          { "id": "n_3", "block": "packsmith/raw-command@1.0.0",
            "inputs": { "command": "effect give @s minecraft:glowing" } }
        ] } }
    ]
  }
}
```

Rules:

1. A child node MUST be a statement node.
2. A child node appears in exactly one slot at exactly one index. There is no other way for a
   statement node to exist in a graph.
3. Index order is execution order. Nothing else expresses order. In particular, node
   coordinates MUST NOT, because they live in `layout.json` and never reach the compiler
   (ADR-0013).
4. A slot MAY be empty. An empty slot is a valid, silent program, not an error.
5. Nesting depth is unbounded in the model. An implementation MAY impose a limit; if it does,
   exceeding it MUST be a diagnostic naming the outermost node, not a stack overflow.

An invalid order is therefore not representable. There is no dangling statement, no statement
with two predecessors, and no cycle in execution.

### 2.2 Statement addresses

Every statement has a stable address: `(node id, slot name, index)`. This is the anchor for
diagnostics about function bodies, and it is what lets a message name the thing the user
placed before it names the file the result landed in (ADR-0009).

### 2.3 Nesting and `execute`

Minecraft's `execute` is a wrapper around one command. In the graph it is an ordinary
statement node whose modifiers are typed input ports and whose target is a slot. Wrapping
existing statements is a move of children into that slot.

How a wrapper with several children lowers -- repeated `execute ... run` lines, or a generated
sub-function -- is decided by the block, not by this document.

### 2.4 Scope of data edges

An input port of a node `N` MAY connect to an output port of a value node `V` when `V` is
reachable from `N` without leaving the enclosing document, and when the resulting data
dependencies are acyclic. Data dependency cycles MUST be rejected with a diagnostic naming
every node on the cycle.

A container node MAY declare **slot-scoped outputs** (for example, the current entity inside
an "as each" wrapper). Such an output is visible only to nodes inside that slot, at any depth.
A data edge referencing it from outside MUST be rejected.

---

## 3. Type references

A type reference is a JSON object with a `type` key naming one of the eleven types below, plus
that type's parameters. There are no type variables, no unions, no user-defined named types,
and no nullability: an optional value is a port marked optional in the manifest, not a
different type.

```json
{ "type": "id", "registry": "minecraft:item" }
{ "type": "list", "of": { "type": "id", "registry": "minecraft:item" } }
```

---

## 4. The types

### 4.1 `bool`

**Means:** yes or no. In the UI, a checkbox or a labelled toggle, never the words `true` and
`false` on their own.

**Literal:** `true` / `false`.

**Connects to:** `bool`.

**Validation:** none beyond the JSON type.

### 4.2 `int`

**Means:** a whole number.

**Parameters:** optional `min` and `max`, inclusive, declared by the block author.

**Literal:** `7`, `-3`.

**Connects to:** `int`, and widens to `float`.

**Validation:** MUST be an integer within the declared bounds, and within the signed 32-bit
range unless the port declares wider bounds, because that is what most game fields hold. A
value outside the bounds MUST be reported with both the bound and the value.

### 4.3 `float`

**Means:** a number that may have a fractional part.

**Parameters:** optional `min` and `max`, inclusive.

**Literal:** `0.5`, `-2`, `3`.

**Connects to:** `float`. An `int` may be connected to a `float` port.

**Validation:** MUST be finite. `NaN` and the infinities are not representable in JSON and are
not accepted in any other spelling. Determinism (ADR-0007) requires that the emitter serialise
floats with the shortest representation that round-trips, and that negative zero be normalised
to zero before it reaches the output.

Prefer `int` where the game field is integral. Every `float` in a graph is a place where two
implementations can disagree about the last digit.

### 4.4 `string`

**Means:** text.

**Parameters:** `format`, a string naming the validator to apply. Defaults to `plain`. The set
of formats is **open** and supplied by target data, in the same way registry categories are
(ADR-0010). There is no closed enum of formats anywhere. A format the target does not define
MUST produce a "not supported for this target" diagnostic, not a parse error. Optional
`max_length`.

Formats defined by v1 target data:

| Format | Meaning | Validated by |
|---|---|---|
| `plain` | arbitrary text | length only |
| `command` | one command line, no leading slash | the target's command grammar (ADR-0012) |
| `selector` | an entity selector | the target's command grammar, entity-argument production |

**Literal:** `"say hello"`, `"@e[type=minecraft:zombie,limit=1]"`.

**Connects to:** a `string` port of the same format, or a `string` port of format `plain`.
A `plain` value MUST NOT flow into a `command` or `selector` port: validation happens where
the format is declared, and allowing the widening backwards would move a real error from
compile time to the game.

**Validation:** `command` and `selector` are handed to the extracted Brigadier tree for the
requested target. This is the machinery ADR-0012 already requires; these formats add no new
subsystem. Diagnostics from it name the game concept before the file, and where the grammar
knows an argument moved between releases, they say so.

**Note.** There is no separate `command` type and no separate `selector` type. See ADR-0016.

### 4.5 `id`

**Means:** a namespaced resource identifier -- the thing written `minecraft:diamond` or
`example:my_function`. This is the type that carries almost all of a data pack's meaning.

**Parameters:**

- `registry`, a namespaced registry key such as `minecraft:item`, `minecraft:block`,
  `minecraft:entity_type`, `minecraft:function`. It is a **string**, looked up in target data.
  There is no enum of registries in this spec, in any schema, or in any implementation
  (ADR-0010).
- `allow_tag`, default `false`. When `true`, the value may instead be a tag reference,
  written with a leading `#`.

**Literal:**

```json
"minecraft:diamond"
"example:my_function"
"#minecraft:planks"
```

A value without a namespace MUST be rejected rather than defaulted. The newcomer who types
`diamond` gets a diagnostic that explains namespaces and offers `minecraft:diamond` as the
fix; silently assuming `minecraft:` teaches nothing and breaks the moment they publish.

**Connects to:** an `id` port with the same `registry`. An `id` whose `allow_tag` is `false`
may connect to one whose `allow_tag` is `true`; the reverse MUST be rejected.

**Validation:** the syntax rule for identifiers as the target defines it. Then, if the target
data table carries a member list for `registry`, membership is checked and an unknown member
MUST produce a diagnostic that names the registry and offers near-miss suggestions. If the
target data carries no member list for that registry -- which is the normal case for ids the
pack itself defines, such as functions and tags -- only syntax is checked, and resolution to a
definition elsewhere in the project is the compiler's own cross-reference pass, not this type.

Tag references are checked against the tag registry corresponding to `registry`, as the target
data table names it.

### 4.6 `block_state`

**Means:** a block, optionally with some of its properties pinned. What `setblock` and the
block conditions of predicates take.

**Literal:**

```json
{ "block": "minecraft:oak_stairs",
  "properties": { "facing": "north", "half": "top" } }
```

`properties` is optional and MAY be partial: an absent property means "any" in a test context
and "the block's default" in a placement context. Which one applies is the consuming block's
business, and its documentation MUST say.

**Connects to:** `block_state`. An `id` of registry `minecraft:block` does **not** connect to
it; a block conversion is a block, not a coercion, because the two mean different things in a
test context.

**Validation:** the block id against the block registry, then each property name against that
specific block's property table and each value against that property's allowed values, both
from the extracted block data (ADR-0014). Property values are strings in this representation,
including `"true"` and `"5"`, because that is how the game writes them.

### 4.7 `item_stack`

**Means:** some quantity of an item, optionally with components.

**Literal:**

```json
{ "item": "minecraft:diamond_sword",
  "count": 1,
  "components": { "minecraft:damage": 4 } }
```

`count` defaults to `1`. `components` defaults to empty.

**Connects to:** `item_stack`. An `id` of registry `minecraft:item` does not connect to it, for
the same reason as `block_state`.

**Validation:** the item id against the item registry. `count` MUST be at least 1; no upper
bound is checked in v1, because the maximum stack size is itself a component default and we do
not extract component defaults (ADR-0014 keeps the subset thin). Each key of `components` MUST
be a syntactically valid id and MUST exist in the target's data component type registry; the
component's **value** is passed through unvalidated. This is a deliberate, documented hole: it
is the one place where a mistake reaches the game instead of the editor.

### 4.8 `text`

**Means:** a text component -- the structured, translatable, styled text the game shows to
players. Advancement titles and descriptions, and the pack description, are text.

**Literal:** any of the shapes the target accepts.

```json
"Hello"
{ "translate": "block.minecraft.stone" }
[ { "text": "Score: ", "color": "gold" }, { "score": { "name": "@s", "objective": "kills" } } ]
```

**Connects to:** `text`. A `string` of format `plain` may connect to a `text` port, and means a
literal, unstyled, untranslated run of text.

**Validation:** shape only -- that the value is a string, an array of text components, or an
object carrying exactly one content field the target recognises. Style keys are not exhaustively
checked and translation keys are not resolved, because neither the language files nor the full
component schema are in the extracted subset.

### 4.9 `list`

**Means:** an ordered, homogeneous collection of values. It is *not* a sequence of statements;
statements live in slots.

**Parameters:** `of`, a type reference. Optional `min_items`, `max_items`, `unique`.

**Literal:** a JSON array of literals of the element type.

```json
[ "minecraft:oak_planks", "minecraft:birch_planks" ]
```

**Connects to:** `list` whose element type is identical. List types are invariant: `list<int>`
does not connect to `list<float>`, even though `int` connects to `float`. Element-wise widening
is a conversion block if anyone ever needs it.

**Validation:** element-wise, plus the declared cardinality. `unique` compares the canonical
form of each element. Order is preserved exactly as written and MUST NOT be sorted by the
compiler, because a list is often a priority order.

### 4.10 `enum`

**Means:** a choice from a fixed set that the **block author** declares -- a comparison
operator, a sort direction, a slot name in a UI sense. It is a per-port authoring convenience,
never a taxonomy of the game.

**Parameters:** `choices`, a non-empty array of distinct ASCII strings.

**Literal:** one of the strings.

```json
"greater_than"
```

**Connects to:** an `enum` with the identical set of choices, in the identical order.

**Validation:** membership in `choices`. A non-member MUST produce a diagnostic listing the
valid choices in full; the set is small by construction, so there is no reason to truncate it.

Display labels for each choice belong to the block manifest's port metadata, not to the type.

**Constraint (ADR-0010).** `enum` MUST NOT be used to model registry categories, pack kinds,
registry contents, block property values, or anything else the game defines. Those are `id`, or
`block_state`, and they are validated against target data. A reviewer seeing an `enum` whose
choices are game concepts should treat it as a bug.

### 4.11 `body`

**Means:** the sequencing type. A slot of type `body` holds an ordered array of child statement
nodes. This is the only type that appears on a slot, and slots are the only place it appears.

**Parameters:** none in v1. There is no way to restrict which statement blocks a slot accepts,
because restricting it would require a taxonomy of statements that we do not have and do not
want; a block that cannot meaningfully appear somewhere reports it as a diagnostic during
lowering.

**Literal:** an array of nodes; see section 2.1. It is not a value, it is not connectable, it
does not appear on an input or output port, and there is no literal for it outside a slot.

**Connects to:** nothing. `body` never participates in a data edge.

**Validation:** every element MUST be a statement node whose block declares support for the
requested target. An element that is a value node MUST be rejected with a diagnostic naming the
slot and the index.

---

## 5. Assignability

An edge from an output of type `S` to an input of type `T` is valid when, and only when, one of
the following holds. There are no other coercions, and implementations MUST NOT add any.

| From | To | Note |
|---|---|---|
| `T` | `T` | identical parameters, compared structurally |
| `int` | `float` | the only numeric widening |
| `string` any format | `string` format `plain` | allowed |
| `string` format `plain` | `text` | literal unstyled text |
| `id` `allow_tag: false` | `id` `allow_tag: true` | same `registry` |

A rejected connection MUST be reported with both types spelled the way the UI spells them
("this wants an *item*, and that gives a *block*"), and, where a conversion block exists, with
that block offered as the fix.

---

## 6. Literals, determinism, and the build hash

- An input port holds a literal or an edge, never both. An optional port with neither uses the
  default declared in the block manifest.
- Literals are ordinary JSON values as described above. There are no embedded expressions, no
  interpolation syntax, and no references written inside strings (ADR-0003). A value that
  depends on another value is an edge.
- Object key order in a literal is not significant, and the compiler MUST canonicalise it
  before hashing. Array order is always significant.
- Number formatting is canonicalised before it reaches the output: integers without a decimal
  point, floats by shortest round-trip, no negative zero, no exponent where a plain form is
  shorter (ADR-0007).
- Slot arrays contribute their order to the build hash. Node coordinates do not exist here at
  all (ADR-0013).

---

## 7. Coverage of the v1 registries

The type set is sized to the seven v1 categories (ADR-0010, OPEN-QUESTIONS A3) and nothing
beyond them.

| Category | Types it needs |
|---|---|
| `function` | `body`, `string(command)`, `string(selector)`, `id` |
| tags | `list<id>`, `id`, `bool` |
| `recipe` | `item_stack`, `id`, `list<id>`, `int`, `float` |
| `loot_table` | `id`, `item_stack`, `int`, `float`, `list` |
| `advancement` | `text`, `id`, `item_stack`, `bool`, `list` |
| `predicate` | `id`, `block_state`, `item_stack`, `int`, `float`, `bool` |
| `item_modifier` | `item_stack`, `id`, `int`, `enum` |

Adding a category (worldgen, resource packs) is a target data update. It is not licence to add
a type. If a future category genuinely needs a twelfth type, that is an amendment to ADR-0016
with the same argument made again: every type is a concept a beginner has to learn and a case
four SDKs have to implement.

---

## 8. Deliberately absent

| Not a type | Instead |
|---|---|
| `command`, `selector` | `string` with a `format` (4.4) |
| a range `{min, max}` | two optional `int` ports on the block |
| `object` / `struct` / free JSON | a block with named ports |
| `nbt` / an NBT path | `string(plain)` today; revisit only with a real block that needs it |
| nullable / `option` | a port marked optional, with a default |
| union / "any" | separate ports, or separate blocks |
| a numeric type per width | `int`, bounded by `min` and `max` |
| `duration`, `position`, `dimension` | `int` and `id`; none of them earn a type in v1 |
| an execution / control-flow type | slots (section 2) |
