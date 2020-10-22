


function App() {

    const [devs, setDevs] = React.useState(
        [
            {"id":197701401,"time":2,"team_login":"teambrbr4","prob":"H","answer":"Yes"},
            {"id":198342704,"time":3,"team_login":"teambrbr41","prob":"H","answer":"Yes"},
            {"id":198320203,"time":3,"team_login":"teambrbr14","prob":"H","answer":"Yes"},
            {"id":198273802,"time":3,"team_login":"teambrbr42","prob":"H","answer":"Yes"},
            {"id":198956310,"time":4,"team_login":"teambrbr26","prob":"H","answer":"Yes"},
            {"id":198740009,"time":4,"team_login":"teambrbr32","prob":"H","answer":"Yes"},
            {"id":198728208,"time":4,"team_login":"teambrbr64","prob":"H","answer":"Yes"},
            {"id":198503107,"time":4,"team_login":"teambrbr54","prob":"H","answer":"Yes"},
            {"id":198475606,"time":4,"team_login":"teambrbr56","prob":"H","answer":"Yes"},
            {"id":198447905,"time":4,"team_login":"teambrbr51","prob":"H","answer":"Yes"}
        ]);
    // const [devs, setDevs] = React.useState([
    //   {
    //     name: "Sabino",
    //     score: 5,
    //   },
    //   {
    //     name: "Emilio",
    //     score: 3,
    //   },
    //   {
    //     name: "Charie Brown",
    //     score: 2,
    //   },
    // ]);
    // const [devs, setDevs] = React.useState([]);
    // React.useEffect(() => {
    //     fetch("http://localhost:3030/runs")
    //         .then(res => res.json())
    //         .then(
    //             (result) => { setDevs(result); },
    //             (error) => {}
    //         )
    // }, []);
  
    const addInDev = (dev) => {
      const i = devs.indexOf(dev);
      const copyDevs = [...devs];
      copyDevs[i].score++;
    //   copyDevs[i].isChanging = true;
      setDevs(copyDevs.sort((a, b) => b.score - a.score));
    };
  
    return (
      <>
        <div border="1" style={{width: "400px"}}>
          {devs.map((dev) => (
              <div
              key={dev.id}
              style={{
                  'border-style': "solid",
                  'border-width' : "1px",
                  padding: '5px',
                  zIndex: dev.id,
                  position: "absolute",
                  top: devs.indexOf(dev) * 40,
                  transition: "1s ease top",
                }}
                >
                <div style={{float: "left", width: '200px',  margin: '5px', 'border-style': "solid"}}>{dev.team_login}</div>
                <div style={{float: "left", margin: '5px', 'border-style': "solid"}}>{dev.prob}</div>
                <div style={{float: "left", margin: '5px', 'border-style': "solid"}}>{dev.answer}</div>
            </div>
          ))}
        </div>
      </>
    );
  }
  
  ReactDOM.render(
    <App />,
    document.getElementById('root')
  );
  
  
  //export default App;
  