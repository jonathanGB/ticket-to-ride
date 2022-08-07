import React from "react";
import "../CSS/PlayerCustom.css";
import PlayerColor from "../GameState/PlayerColor";

class PlayerCustom extends React.Component<
  { name: String; color: PlayerColor; isSelf: boolean },
  {}
> {
  constructor(props: any) {
    super(props);
  }

  render() {
    let className;
    if (this.props.color) {
      className = `playerColor-${this.props.color}`;
    } else {
      className = "playerColor-default";
    }

    return (
      <div className={className} id="playerCustom">
        <div className="container">{this.props.name}</div>
      </div>
    );
  }
}

export default PlayerCustom;
