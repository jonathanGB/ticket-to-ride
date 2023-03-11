import React from "react";
import * as Styles from "./LandingPage.styles"
import TrainAnimation from "./TrainAnimation";


class LandingPage extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return (
      <div className={Styles.landingPageStyle}>
        <h1 className={Styles.h1Style}>Ticket to Ride</h1>
        <div className={Styles.buttonWrapperStyle}>
          <form action="/create" method="post" className={
            Styles.formStyle}>
            <input
              className={Styles.createButtonStyle}
              type="submit"
              value="All aboard!"
            />
          </form>
        </div>

        <div className={Styles.trainStyle}>
        <TrainAnimation />
        </div>
      </div>
    );
  }
}

export default LandingPage;
