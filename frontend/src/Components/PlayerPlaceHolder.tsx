import React from "react";
import "../Styling/PlayerPlaceHolder.styles.tsx";

class PlayerPlaceHolder extends React.Component<{}, {}> {
  constructor(props: {} | {}) {
    super(props);
  }

  render() {
    return <div className="personicon">
          </div> 
  }
}

export default PlayerPlaceHolder;