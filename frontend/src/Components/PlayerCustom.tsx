import React from 'react';
import '../CSS/PlayerCustom.css';

class PlayerCustom extends React.Component<{[name: string]: string;}, {}> {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="PlayerCustom-header">
      <h3>{this.props.name} </h3>
    </div>
  }
};

export default PlayerCustom;