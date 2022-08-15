import React from "react";
import { Routes, Route } from "react-router-dom";

import LandingPage from "./LandingPage";
import Game from "./Game";

const Main = () => {
  return (
    <div className="MainHeader">
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
