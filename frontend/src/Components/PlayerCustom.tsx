import React from 'react';
import '../CSS/PlayerCustom.css';
import PlayerColor from '../PlayerColorEnum'

class PlayerCustom extends React.Component<{name: string, color?: PlayerColor}, {}> {
  constructor(props: any) {
    super(props);
  }

  render() {
    let className;
    if(this.props.color){
      className = `playerColor-${this.props.color}`;
    }
    else{
      className = 'playerColor-default';
    }
    console.log(className);
    return <div className={className} id="playerColor">
      <h3>{this.props.name} </h3>
    </div>
  }
};

export default PlayerCustom;