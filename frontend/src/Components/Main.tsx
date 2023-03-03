import React from "react";
import { Routes, Route } from "react-router-dom";
import * as Styles from "./Main.styles"
import LandingPage from "./LandingPage";
import Game from "./Game";

const Main = () => {
  return (
    <div className={Styles.mainStyle}>
      <Routes>
      {" "}
      {/* The Switch decides which component to show based on the current URL.*/}
      <Route path="/" element={<LandingPage />} />
      <Route path="game/*" element={<Game />} />
    </Routes>
    </div>
  );
};

export default Main;
