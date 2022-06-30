import React from 'react';
import '../CSS/LandingPage.css';

class LandingPage extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="CreateGame-header">
      <h1>hello world</h1>
    <header>
    <form action="/create" method="post">
      <input className="create-button" type="submit" value="Create game" />
    </form>
    </header>
    </div>
  }
};

export default LandingPage;
