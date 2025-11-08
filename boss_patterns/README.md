# Boss Pattern JSON Files

This directory contains JSON files that define boss attack and movement patterns.

## File Structure

Each JSON file should follow this structure:

```json
{
  "attack": {
    "type": "SingleShot",
    "cooldown": 1.5,
    "projectile_speed": 400.0
  },
  "movement": {
    "type": "HorizontalPatrol",
    "left_bound": 250.0,
    "right_bound": 350.0,
    "speed": 100.0
  }
}
```

## Attack Pattern Types

### None
Boss doesn't attack.
```json
{
  "type": "None"
}
```

### SingleShot
Simple single shot at player.
```json
{
  "type": "SingleShot",
  "cooldown": 1.5,
  "projectile_speed": 400.0
}
```

### TripleShot
Three projectiles with spread angle.
```json
{
  "type": "TripleShot",
  "cooldown": 2.0,
  "projectile_speed": 350.0,
  "spread_angle": 30.0
}
```

### RapidFire
Burst fire pattern.
```json
{
  "type": "RapidFire",
  "cooldown": 3.0,
  "projectile_speed": 400.0,
  "burst_count": 5,
  "burst_delay": 0.1
}
```

### Sequence
Sequence of actions (for complex patterns).
```json
{
  "type": "Sequence",
  "actions": [
    {
      "action_type": "shoot",
      "direction": {"x": 1.0, "y": 0.0},
      "count": 3,
      "delay": 0.2
    },
    {
      "action_type": "wait",
      "delay": 1.0
    }
  ],
  "loop_pattern": true
}
```

## Movement Pattern Types

### Stationary
Boss doesn't move.
```json
{
  "type": "Stationary"
}
```

### HorizontalPatrol
Moves left/right between bounds.
```json
{
  "type": "HorizontalPatrol",
  "left_bound": 250.0,
  "right_bound": 350.0,
  "speed": 100.0
}
```

### VerticalPatrol
Moves up/down between bounds.
```json
{
  "type": "VerticalPatrol",
  "top_bound": -150.0,
  "bottom_bound": -198.0,
  "speed": 80.0
}
```

### Circular
Circular movement pattern.
```json
{
  "type": "Circular",
  "center": {"x": 300.0, "y": -198.0},
  "radius": 50.0,
  "speed": 1.0
}
```

### Waypoint
Moves between waypoints.
```json
{
  "type": "Waypoint",
  "waypoints": [
    {"x": 250.0, "y": -198.0},
    {"x": 350.0, "y": -198.0},
    {"x": 300.0, "y": -150.0}
  ],
  "speed": 120.0,
  "loop_path": true
}
```

## Loading Patterns

Patterns can be loaded programmatically using the `BossPatternRegistry`:

```rust
let json_content = std::fs::read_to_string("boss_patterns/my_boss.json")?;
boss_pattern_registry.load_from_json("my_boss".to_string(), &json_content)?;
```

Or you can extend the system to load from the `assets` folder using Bevy's asset system.

