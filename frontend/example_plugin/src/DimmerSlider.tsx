import { createSignal, onMount, type Component } from 'solid-js';
import { Slider } from '@ark-ui/solid';
import type { PluginElementProps, WatchUpdate } from '@igloo/types';

interface DimmerSliderProps extends PluginElementProps {
  device_filter?: any;
  entity_filter?: any;
  title?: string;
}

const DimmerSlider: Component<DimmerSliderProps> = (props) => {
  const [value, setValue] = createSignal([0.5]);
  const [isUpdating, setIsUpdating] = createSignal(false);

  onMount(() => {
    props.api.sub(
      {
        Component: {
          device_filter: {
            ...props.device_filter,
          },
          entity_filter: {
            ...props.entity_filter,
          },
          component: "Dimmer",
          post_op: "Mean",
        }
      },
      (update: WatchUpdate) => {
        if ('ComponentAggregate' in update && 'Real' in update.ComponentAggregate) {
          setValue([update.ComponentAggregate.Real]);
        } else if ('Component' in update && 'Real' in update.Component) {
          setValue([update.Component.Real]);
        }
      }
    );
  });

  const handleChange = async (newValue: number[]) => {
    setValue(newValue);
    setIsUpdating(true);

    try {
      await props.api.eval({
        Component: {
          device_filter: {
            ...props.device_filter,
          },
          entity_filter: {
            ...props.entity_filter,
          },
          action: {
            Set: { Real: newValue[0] }
          },
          component: "Dimmer",
        }
      });
    } catch (error) {
      console.error('[DimmerSlider] Failed to set value:', error);
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <div style={{
      padding: '20px',
      border: '1px solid #e5e7eb',
      'border-radius': '8px',
    }}>
      {props.title && <h3 style={{ margin: '0 0 16px 0' }}>{props.title}</h3>}

      <Slider.Root
        min={0}
        max={1}
        step={0.01}
        value={value()}
        onValueChange={(e) => setValue(e.value)}
        onValueChangeEnd={(e) => handleChange(e.value)}
      >
        <Slider.Label>
          Brightness: {Math.round(value()[0] * 100)}%
        </Slider.Label>

        <Slider.Control>
          <Slider.Track style={{
            height: '8px',
            background: '#e5e7eb',
            'border-radius': '4px',
            position: 'relative',
          }}>
            <Slider.Range style={{
              background: isUpdating() ? '#9ca3af' : '#3b82f6',
              height: '100%',
              'border-radius': '4px',
            }} />
          </Slider.Track>

          <Slider.Thumb
            index={0}
            style={{
              width: '24px',
              height: '24px',
              background: 'white',
              border: '2px solid #3b82f6',
              'border-radius': '50%',
              cursor: 'pointer',
              'box-shadow': '0 2px 4px rgba(0,0,0,0.1)',
            }}
          />
        </Slider.Control>
      </Slider.Root>
    </div>
  );
};

export default DimmerSlider;
