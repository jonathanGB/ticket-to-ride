import React from "react";
import "./TrainAnimation.css";

class TrainAnimation extends React.Component<{}, {}> {
  constructor(props: any) {
    super(props);
  }

  render() {
    return (
      <div className="train">
        <div className="background" />
      </div>
    );
  }
}

export default TrainAnimation;
