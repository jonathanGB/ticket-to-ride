import { mergeStyles } from "@fluentui/react";
import React from "react";
import MapSVG from '../Svg/MapSVG';
class Map extends React.Component {
  constructor(props: any) {
    super(props);
  }
  screenWidth = window.innerWidth;
  screenHeight = window.innerHeight;
  mapDivStyle = {
    position: "fixed" as const, // Using 'as const' for literal types
    top: "0px",
    left: "0px",
    height: "1080px",
    width: "1920px",
    minWidth: "1920px", /* Adjust as needed */
    minHeight: "1080px", /* Adjust as needed */
    textAlign: "center" as const, // Again, if TypeScript requires specific typing for textAlign
    zIndex: 0 // zIndex as number is usually fine
};
  render() {
    return (
      <div className="Map" style={this.mapDivStyle}>
          <MapSVG />
      </div>
    );
  }
}

export default Map;
