import { mergeStyleSets, mergeStyles } from '@fluentui/merge-styles';
import PlayerColor from "../GameState/PlayerColor";

export const playerCustomStyle = mergeStyleSets({
  playerCustomStyle: mergeStyles({
    borderStyle: "solid",
    borderRadius: "1rem",
    width: "200px",
    height: "200px",
    margin: "5vh 5vh",
    textAlign: "center",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    color: "white",
    fontFamily: "sans-serif",
    fontSize: "36px",
    fontWeight: "400px"
  }),
  playerCustomSelfStyle: mergeStyles({
    borderStyle: "solid",
    borderRadius: "1rem",
    width: "100%",
    height: "100%",
    textAlign: "center",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    color: "white",
    fontFamily: "sans-serif",
    fontSize: "36px",
    fontWeight: "400px"
  }),
  inputStyle: mergeStyles({
    backgroundColor: "#81a6d8",
    borderColor: "#81a6d8",
    boxShadow: "none",
    borderStyle: "none"
  }),
});

export const getCSSColorHexCode = (color: string): string  => {
  switch (color.toLowerCase()) {
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