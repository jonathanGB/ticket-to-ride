import React from 'react';
import '../CSS/LandingPage.css';

class LandingPage extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="LandingPage-header">
      <h1>Ticket to Ride</h1>
      <div className='train'>
        <div className='background'>
        </div>
        <div className="demo-content">
            <form className="buttonForm" action="/create" method="post">
              <input className="create-button" type="submit" value="Create game" />
            </form>
          </div>
      </div>
    </div>
  }
};

export default LandingPage;
