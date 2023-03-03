import React from "react";
import * as Styles from "./App.styles"
import Main from "./Main";

class App extends React.Component {
  constructor(props: {} | Readonly<{}>) {
    super(props);
  }
  render() {
    return (
      <div className={Styles.appStyle}>
        <Main />
      </div>
    );
  }
}

export default App;
