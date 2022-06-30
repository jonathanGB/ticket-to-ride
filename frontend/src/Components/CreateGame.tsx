import React from 'react';
import './CSS/CreateGame.css';

class CreateGame extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return <div className="CreateGame-header">
    <header>
    <form action="/create" method="post">
      <input className="create-button" type="submit" value="Create game" />
    </form>
    </header>
    </div>
  }
};

export default CreateGame;
