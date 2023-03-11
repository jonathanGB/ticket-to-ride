import { mergeStyleSets, mergeStyles } from '@fluentui/merge-styles';

export const lobbyStyle = mergeStyleSets({
  lobbyHeaderStyle: mergeStyles({
    backgroundColor: "#aec8eb",
    minHeight: "100vh",
    minWidth: "100vw",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "space-between",
    position: "relative"
  }),
  playerSectionsStyle: mergeStyles({
    width: "100vw",
    minWidth: "100vw",
    display: "flex",
    flexDirection: "row",
    justifyContent: "space-between"
  }),
  trainContainerStyle: mergeStyles({
    margin: 0
  }),
  otherPlayersHeaderStyle: mergeStyles({
    width: "40vw",
    minWidth: "40vw",
    display: "flex",
    flexDirection: "row",
    justifyContent: "center",
    alignItems: "center",
    flexWrap: "wrap"
  }),
  selfPlayerSectionStyle: mergeStyles({
    width: "30vw",
    minWidth: "30vw",
    display: "flex",
    flexDirection: "column",
    justifyContent: "center",
    alignItems: "center"
  }),
  selfPlayerHeaderStyle: mergeStyles({
    width: "30vw",
    minWidth: "30vw",
    height: "20vh",
    minHeight: "20vh",
    marginBottom: "3vh"
  }),
  inputStyle: mergeStyles({
    marginBottom: "3vh",
    height: "5vh",
    width: "100%",
    fontSize: "2rem"
  }),
  textFieldStyle: mergeStyles({
    width: "20vw",
    backgroundColor: "#D9D9D9",
    color: "#3e4756",
    fontWeight: "lighter"
  }),
  submitButtonStyle: mergeStyles({
    width: "10vw",
    color: "white"
  }),
  playerChoiceContainerStyle: mergeStyles({
    width: "30vw",
    color: "white"
  }),
});



// ::placeholder {
//   color: #A2ACBD;
// }

