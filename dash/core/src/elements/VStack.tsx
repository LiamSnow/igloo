import type { PluginComponent } from '@igloo/types';

interface VStackProps {
  gap?: string;
  padding?: string;
  align?: 'start' | 'center' | 'end' | 'stretch';
  [key: string]: any;
}

const VStack: PluginComponent<VStackProps> = (props) => {
  return (
    <div
      style={{
        display: 'flex',
        'flex-direction': 'column',
        gap: props.gap || '16px',
        padding: props.padding || '0',
        'align-items': props.align || 'stretch',
      }}
    >
      {props.body}
    </div>
  );
};

export default VStack;
