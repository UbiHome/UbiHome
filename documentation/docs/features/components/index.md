# Components

Components make data or functionality available for external systems via [Connectivity Features](../connectivity/index.md), or for reuse in other components.

[Platforms](../platforms/index.md) provide the implementation for component behavior.

A component entry is defined under a component key (for example `sensor:` or `switch:`), and each entry selects a `platform`.

## Common Component Attributes

All components share a set of [common attributes](./base.md) that can be used to customize their behavior.

## Components

<div class="grid cards" markdown>

-   :material-motion-sensor:{ .lg .middle } [**Binary Sensor**](./binary_sensor.md)

    ---

    Track on/off states or occupancy.

    [:octicons-arrow-right-24: View Documentation](./binary_sensor.md)

-   :material-button-pointer:{ .lg .middle } [**Button**](./button.md)

    ---

    Trigger an action on the device.

    [:octicons-arrow-right-24: View Documentation](./button.md)

-   :material-tune-vertical:{ .lg .middle } [**Number**](./number.md)

    ---

    Set or read numeric values with min/max/step constraints.

    [:octicons-arrow-right-24: View Documentation](./number.md)

-   :material-thermometer:{ .lg .middle } [**Sensor**](./sensor.md)

    ---

    Make data available as a sensor.

    [:octicons-arrow-right-24: View Documentation](./sensor.md)

-   :material-toggle-switch-outline:{ .lg .middle } [**Switch**](./switch.md)

    ---

    Switch something on or off.

    [:octicons-arrow-right-24: View Documentation](./switch.md)

</div>

## Component Features

<div class="grid cards" markdown>

-   :material-graph-outline:{ .lg .middle } [**Actions**](./actions.md)

    ---

    Trigger actions from state changes. This even works offline!

    [:octicons-arrow-right-24: View Documentation](./actions.md)

-   :material-function:{ .lg .middle } [**Filters**](./filters.md)

    ---

    Modify component values (for example `round`, `delayed_on`).

    [:octicons-arrow-right-24: View Documentation](./filters.md)

</div>
