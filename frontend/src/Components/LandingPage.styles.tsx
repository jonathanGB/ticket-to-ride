import { mergeStyles } from '@fluentui/react';

export const landingPageStyle = mergeStyles({
  textAlign: "center",
  backgroundColor: "#AEC8EB",
  minHeight: "100vh",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "space-between",
  overflow: "hidden",
  position: "relative"
});

export const formStyle = mergeStyles({
  display: "flex",
  flexDirection: "row",
  alignItems: "center",
  justifyContent: "center"
});

export const buttonWrapperStyle = mergeStyles({
  width: "100%",
  height: "256px",
  position: "relative",
  display: "flex",
  alignItems: "center",
  justifyContent: "center"
});

export const h1Style = mergeStyles({
  fontSize: "10rem",
  margin: "0em",
  marginTop: "10vh",
  color: "#FCFBF4",
  stroke: "2px #3A689B"
});

export const createButtonStyle = mergeStyles({
  backgroundColor: "#3A689B",
  background: "linear-gradient(to top right, #2D537A 0%, #3A689B 50%, #2D537A 100%)",
  color: "#FCFBF4",
  width: "200px",
  minWidth: "200px",
  height: "8vh",
  fontSize: "1.8rem",
  fontWeight: "bold",
  border: "none",
  borderRadius: "8em",
  cursor: "pointer",
  position: "relative",
  zIndex: 1,
  selectors: {
    ':hover': {
      background: "linear-gradient(to top right, #2D537A 15%, #3A689B 50%, #2D537A 85%)"
    }
  }
})

export const trainStyle =  mergeStyles({
  marginBottom: "5vh"
})
