import React from 'react';
import '../CSS/LandingPage.css';
import TrainAnimation from './TrainAnimation';

class LandingPage extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="LandingPage-header">
      <h1>Ticket to Ride</h1>

      <div className="button-wrapper">
        <form className="buttonForm" action="/create" method="post">
          <input className="create-button" type="submit" value="All aboard!" />
        </form>
      </div>

    <TrainAnimation/>
    </div>
  }
};

export default LandingPage;
