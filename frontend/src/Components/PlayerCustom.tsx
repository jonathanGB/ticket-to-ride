import React from 'react';
import '../CSS/PlayerCustom.css';
import PlayerColor from '../GameState/PlayerColor'

class PlayerCustom extends React.Component<{name: String, color: PlayerColor, isSelf: boolean}, {}> {
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

    if(this.props.isSelf){
      return <div className={className} id="playerCustomSelf">
        <div>{this.props.name} </div>
    </div>
    }
    return <div className={className} id="playerCustomOther">
      <div>{this.props.name} </div>
    </div>
  }
};

export default PlayerCustom;