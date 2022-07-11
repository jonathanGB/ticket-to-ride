import React from 'react';
import '../CSS/PlayerCustom.css';

class PlayerCustom extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="PlayerCustom-header">
      <h1>Hello game </h1>
    </div>
  }
};

export default PlayerCustom;