# Components

Components make data or functionality available for external systems via [Connectivity Features](../connectivity/index), or for reuse in other components.

[Platforms](../platforms/index) provide the implementation for component behavior.

A component entry is defined under a component key (for example `sensor:` or `switch:`), and each entry selects a `platform`.

## Common Component Attributes

All components share a set of [common attributes](./base) that can be used to customize their behavior.

## Components

<div class="grid cards" markdown>

-   :material-motion-sensor:{ .lg .middle } [**Binary Sensor**](./binary_sensor)

    ---

    Track on/off states or occupancy.

    [:octicons-arrow-right-24: View Documentation](./binary_sensor)

-   :material-button-pointer:{ .lg .middle } [**Button**](./button)

    ---

    Trigger an action on the device.

    [:octicons-arrow-right-24: View Documentation](./button)

-   :material-toggle-switch-outline:{ .lg .middle } [**Switch**](./switch)

    ---

    Switch something on or off.

    [:octicons-arrow-right-24: View Documentation](./switch)

-   :material-tune-vertical:{ .lg .middle } [**Number**](./number)

    ---

    Set or read numeric values with min/max/step constraints.

    [:octicons-arrow-right-24: View Documentation](./number)

-   :material-thermometer:{ .lg .middle } [**Sensor**](./sensor)

    ---

    Make data available as a sensor.

    [:octicons-arrow-right-24: View Documentation](./sensor)

-   :material-form-textbox:{ .lg .middle } [**Text Sensor**](./text_sensor)

    ---

    Make text values available as a sensor.

    [:octicons-arrow-right-24: View Documentation](./text_sensor)

-   :material-toggle-switch-outline:{ .lg .middle } [**Switch**](./switch)

    ---

    Switch something on or off.

    [:octicons-arrow-right-24: View Documentation](./switch)

</div>

## Component Features

<div class="grid cards" markdown>

-   :material-graph-outline:{ .lg .middle } [**Actions**](./actions)

    ---

    Trigger actions from state changes. This even works offline!

    [:octicons-arrow-right-24: View Documentation](./actions)

-   :material-function:{ .lg .middle } [**Filters**](./filters)

    ---
    
    Modify component values (for example `round`, `delayed_on`).

    [:octicons-arrow-right-24: View Documentation](./filters)

</div>
