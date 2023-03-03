import React from "react";
import * as Styles from "./ColorPicker.styles";
import { getCSSColorHexCode } from "./PlayerCustom.styles";
import PlayerColor from "../GameState/PlayerColor";

class ColorPicker extends React.Component<
  { selfColor?: PlayerColor; onChange: any },
  {}
> {
  constructor(props: any) {
    super(props);
  }
  // TODO: make each color its own component to add X's
  render() {
    let inputList = new Array();
    for(const color in PlayerColor){
      let inputStyle={
        borderColor: getCSSColorHexCode(color),
        backgroundColor: getCSSColorHexCode(color),
        Color: getCSSColorHexCode(color),
        borderStyle: "double",
        borderRadius: "1rem",
        width: "10vh",
        height: "10vh",
        margin: "1vh"
      }
      inputList.push(
        <input
          style={inputStyle}
          id={color}
          onClick={this.props.onChange}
          value={color}
        />
      )
    }
    return (
      <div className={Styles.colorPickerStyle}>
        {inputList}
      </div>
    );
  }
}

export default ColorPicker;
