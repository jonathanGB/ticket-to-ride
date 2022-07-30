import React from 'react';
import '../CSS/PlayerCustom.css';
import PlayerColor from '../PlayerColorEnum'

class PlayerCustom extends React.Component<{name?: string, color?: PlayerColor, input?: JSX.Element}, {}> {
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

    if(this.props.input){
      return <div className={className} id="playerCustomSelf">
      <div>{this.props.input} </div>
    </div>
    }
    return <div className={className} id="playerCustomOther">
      <div>{this.props.name} </div>
    </div>
  }
};

export default PlayerCustom;