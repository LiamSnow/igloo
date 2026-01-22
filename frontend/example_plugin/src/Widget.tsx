import { createSignal, type Component } from 'solid-js';
import { Slider } from '@ark-ui/solid';

const Widget: Component = () => {
  const [count, setCount] = createSignal(0);

  return (
    <div style={{
      padding: '20px',
      border: '2px solid #3b82f6',
      'border-radius': '8px',
      margin: '20px 0'
    }}>
      <h2>Example Plugin Widget</h2>
      
      <div style={{ margin: '20px 0' }}>
        <button 
          onClick={() => setCount(count() + 1)}
          style={{
            padding: '10px 20px',
            background: '#3b82f6',
            color: 'white',
            border: 'none',
            'border-radius': '4px',
            cursor: 'pointer'
          }}
        >
          Clicked {count()} times
        </button>
      </div>

      <div style={{ margin: '20px 0' }}>
        <Slider.Root 
          defaultValue={[5, 10]}
          style={{
            "max-width": "320px",
            width: "100%",
            display: "flex",
            "flex-direction": "column"
          }}
        >
          <Slider.Label>Label</Slider.Label>
          <Slider.ValueText style={{ "margin-inline-start": "12px" }} />
          <Slider.Control
            style={{
              "--slider-thumb-size": "20px",
              "--slider-track-height": "4px",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
              position: "relative",
              height: "var(--slider-thumb-size)"
            }}
          >
            <Slider.Track
              style={{
                background: "rgba(0, 0, 0, 0.2)",
                "border-radius": "9999px",
                height: "var(--slider-track-height)",
                width: "100%"
              }}
            >
              <Slider.Range
                style={{
                  background: "magenta",
                  "border-radius": "inherit",
                  height: "100%"
                }}
              />
            </Slider.Track>
            <Slider.Thumb 
              index={0}
              style={{
                width: "var(--slider-thumb-size)",
                height: "var(--slider-thumb-size)",
                "border-radius": "999px",
                background: "white",
                "box-shadow": "rgba(0, 0, 0, 0.14) 0px 2px 10px"
              }}
            >
              <Slider.HiddenInput />
            </Slider.Thumb>
            <Slider.Thumb 
              index={1}
              style={{
                width: "var(--slider-thumb-size)",
                height: "var(--slider-thumb-size)",
                "border-radius": "999px",
                background: "white",
                "box-shadow": "rgba(0, 0, 0, 0.14) 0px 2px 10px"
              }}
            >
              <Slider.HiddenInput />
            </Slider.Thumb>
          </Slider.Control>
        </Slider.Root>
      </div>
    </div>
  );
};

export default Widget;
