


function App() {

    const [devs, setDevs] = React.useState([]);
  
    const addInDev = (dev) => {
      const i = devs.indexOf(dev);
      const copyDevs = [...devs];
      copyDevs[i].score++;
    //   copyDevs[i].isChanging = true;
      setDevs(copyDevs.sort((a, b) => b.score - a.score));
    };
    
    setInterval(() => {
      fetch("http://localhost:3030/runs")
      .then(res => res.json())
      .then(
        (result) => {
          console.log("Obtained results from api.")
          setDevs(result);
        },
        // Nota: É importante lidar com os erros aqui
        // em vez de um bloco catch() para não recebermos
        // exceções de erros dos componentes.
        (error) => {
          console.log("ajax request error:", error);
          // this.setState({
          //   isLoaded: true,
          //   error
          // });
        }
      )
    }, 10000);

    return (
      <>
        <div border="1">
          {devs.map((dev) => (
              <div className="run"
              key={dev.id}
              style={{
                  zIndex: dev.id,
                  position: "absolute",
                  top: 10 + devs.indexOf(dev) * 50,
                  transition: "1s ease top",
                }}
                >
                <div className="cell login">{dev.team_login}</div>
                <div className="cell problem">{dev.prob}</div>
                <div className="cell answer">{dev.answer}</div>
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
  