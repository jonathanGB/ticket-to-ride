import React from "react";
import "../CSS/PlayerCustom.css";
import PlayerColor from "../GameState/PlayerColor";

class PlayerCustom extends React.Component<
  { name: String; color: PlayerColor; isSelf: boolean, isReady: boolean },
  {}
> {
  constructor(props: any) {
    super(props);
  }
  getCSSColor = (): string  => {
    switch (this.props.color) {
      case PlayerColor.Black: {
        return "#3e4756"
      }
      case PlayerColor.Green: {
        return "#33c191"
      }
      case PlayerColor.Blue: {
        return "#28b7e5"
      }
      case PlayerColor.Orange: {
        return "#db9039"
      }
      case PlayerColor.Pink: {
        return "#cf8298"
      }
      case PlayerColor.Red: {
        return "#b23a48"
      }
      case PlayerColor.White: {
        return "#f0e1d1"
      }
      case PlayerColor.Yellow: {
        return "#dbda60"
      }
      default: {
        return "#FFFFFF"
      }
    }
  }
  render() {
    let divStyle;
    let colorHexCode = this.getCSSColor()
    if(this.props.isReady){
      divStyle={
        borderColor: "#F8FC4F",
        borderWidth: "0.25rem",
        backgroundColor: colorHexCode
      }
    }
    else{
      divStyle={
        borderColor: colorHexCode,
        borderWidth: "0.25rem",
        backgroundColor: colorHexCode
      }
    }
    if(this.props.isSelf){
      return (
        <div id="playerCustomSelf" style={divStyle}>
          <div className="container">{this.props.name}</div>
        </div>
      );
    }
    return (
      <div id="playerCustom" style={divStyle}>
        <div className="container">{this.props.name}</div>
      </div>
    );
  }
}

export default PlayerCustom;
