import React from "react";
import * as Styles from "./PlayerCustom.styles"
import PlayerColor from "../GameState/PlayerColor";

class PlayerCustom extends React.Component<
  { name: String; color: PlayerColor; isSelf: boolean, isReady: boolean },
  {}
> {
  constructor(props: any) {
    super(props);
  }
  render() {
    let divStyle;
    let colorHexCode = Styles.getCSSColorHexCode(this.props.color)
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
        <div className={Styles.playerCustomStyle.playerCustomSelfStyle} style={divStyle}>
          <div>{this.props.name}</div>
        </div>
      );
    }
    return (
      <div id={Styles.playerCustomStyle.playerCustomStyle} style={divStyle}>
        <div>{this.props.name}</div>
      </div>
    );
  }
}

export default PlayerCustom;
